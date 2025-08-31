use std::ffi::{CStr, CString};
use std::ptr;

#[repr(C)]
pub struct SdBus {
    _private: [u8; 0],
}

#[repr(C)]
pub struct SdBusMessage {
    _private: [u8; 0],
}

// see https://github.com/systemd/systemd/blob/main/src/systemd/sd-bus-protocol.h#L118
#[repr(C)]
pub struct SdBusError {
    pub name: *const i8,
    pub message: *const i8,
    _need_free: i32,
}

impl Default for SdBusError {
    fn default() -> Self {
        SdBusError {
            name: ptr::null(),
            message: ptr::null(),
            _need_free: 0,
        }
    }
}

#[link(name = "systemd")]
unsafe extern "C" {
    unsafe fn sd_bus_default_system(bus: *mut *mut SdBus) -> i32;
    unsafe fn sd_bus_unref(bus: *mut SdBus) -> *mut SdBus;
    unsafe fn sd_bus_call_method(
        bus: *mut SdBus,
        destination: *const i8,
        path: *const i8,
        interface: *const i8,
        member: *const i8,
        error: *mut SdBusError,
        reply: *mut *mut SdBusMessage,
        types: *const i8,
        interactive: i32,
    ) -> i32;
    unsafe fn sd_bus_message_unref(message: *mut SdBusMessage) -> *mut SdBusMessage;
    unsafe fn sd_bus_error_free(error: *mut SdBusError);
    unsafe fn sd_bus_message_enter_container(message: *mut SdBusMessage, type_: u8, contents: *const i8) -> i32;
    unsafe fn sd_bus_message_exit_container(message: *mut SdBusMessage) -> i32;
    unsafe fn sd_bus_message_at_end(message: *mut SdBusMessage, complete: i32) -> i32;
}

//#############################################################################
// The following functions basically implement systemd dbus calls.
// You can check the signatures of systemd dbus interface by using
// `busctl introspect org.freedesktop.login1 /org/freedesktop/login1 org.freedesktop.login1.Manager`
//#############################################################################

/// Suspends the system using systemd.
pub fn systemd_suspend(shutdown_instead: bool) -> Result<(), String> {
    unsafe {
        // first we get a dbus-reference so we can talk with systemd
        let mut bus: *mut SdBus = ptr::null_mut();
        let return_code = sd_bus_default_system(&mut bus);
        if return_code < 0 {
            return Err(format!("Failed to get system bus, return-code: {}", return_code));
        }

        // then we prepare a couple of strings we need for talking with systemd
        let dbus_destination = CString::new("org.freedesktop.login1").unwrap();
        let dbus_path = CString::new("/org/freedesktop/login1").unwrap();
        let dbus_interface = CString::new("org.freedesktop.login1.Manager").unwrap();
        let dbus_member = CString::new(if shutdown_instead {"Poweroff"} else {"Suspend"}).unwrap();
        let dbus_types = CString::new("b").unwrap(); // boolean parameter

        // and prepare some more structs that we need for getting the results of systemd
        let mut call_error = SdBusError::default();
        let mut reply: *mut SdBusMessage = ptr::null_mut();
        
        // call Suspend method, signature: Suspend(b) -> void
        let return_code = sd_bus_call_method(
            bus,
            dbus_destination.as_ptr(),
            dbus_path.as_ptr(),
            dbus_interface.as_ptr(),
            dbus_member.as_ptr(),
            &mut call_error,
            &mut reply,
            dbus_types.as_ptr(),
            1, // interactive=true - so if other processes block suspend, we respect that.
        );
        
        // we don't care for the reply, but if we received one, we have to clean it up.
        if !reply.is_null() {
            sd_bus_message_unref(reply);
        }

        // if the suspend-call failed, we collect the error information and return it.
        if return_code < 0 {
            // collect the error-message
            let error_msg = if !call_error.message.is_null() {
                CStr::from_ptr(call_error.message).to_string_lossy().into_owned()
            } else {
                format!("D-Bus call failed: {}", return_code)
            };

            // clean up all references, so we don't leak any memory
            sd_bus_error_free(&mut call_error);
            sd_bus_unref(bus);
            
            // of the message contains "already in progress" the system tries to suspend - we treat that as success.
            if error_msg.contains("already in progress") {
                return Ok(())    
            }
            // else return the error
            return Err(error_msg);
        }
        // else the call was successful, so we clean up and return Ok.
        sd_bus_error_free(&mut call_error);
        sd_bus_unref(bus);
        
        Ok(())
    }
}

/// Asks systemd whether the system has any active user-sessions. 
pub fn has_active_user_sessions() -> Result<bool, String> {
    unsafe {
        // first we get a dbus-reference so we can talk with systemd
        let mut bus: *mut SdBus = ptr::null_mut();
        let return_code = sd_bus_default_system(&mut bus);
        if return_code < 0 {
            return Err(format!("Failed to get system bus, return-code: {}", return_code));
        }
        
        // prepare strings for D-Bus call to ListSessions
        let dbus_destination = CString::new("org.freedesktop.login1").unwrap();
        let dbus_path = CString::new("/org/freedesktop/login1").unwrap();
        let dbus_interface = CString::new("org.freedesktop.login1.Manager").unwrap();
        let dbus_member = CString::new("ListSessions").unwrap();
        // ListSessions has no parameters, so we omit dbus_types and just send a null-ptr

        // prepare structs for call results
        let mut call_error = SdBusError::default();
        let mut reply: *mut SdBusMessage = ptr::null_mut();

        // call ListSessions method, signature: ListSessions() -> a(susso)
        let return_code = sd_bus_call_method(
            bus,
            dbus_destination.as_ptr(),
            dbus_path.as_ptr(),
            dbus_interface.as_ptr(),
            dbus_member.as_ptr(),
            &mut call_error,
            &mut reply,
            ptr::null(), // no input parameters
            0 // we don't send params, but we need to send anything, so we just send 0
        );
        
        // if the listsessions-call failed, we collect the error information and return it.
        if return_code < 0 {
            let error_msg = if !call_error.message.is_null() {
                CStr::from_ptr(call_error.message).to_string_lossy().into_owned()
            } else {
                format!("D-Bus call failed: {}", return_code)
            };

            // clean up all references, so we don't leak any memory
            sd_bus_error_free(&mut call_error);
            sd_bus_unref(bus);
            // and return the error message
            return Err(error_msg);
        }
        
        // else it was successful, so parse the reply to check if array has any elements
        let has_sessions = if !reply.is_null() {
            // enter the array container (type 'a' = array)
            let array_contents = CString::new("(susso)").unwrap();
            // basically, the array_contents tells how the structures in the array look like, and `b'a'`
            // tells that we want to parse an array-container from reply.
            let enter_result = sd_bus_message_enter_container(reply, b'a', array_contents.as_ptr());
            
            // if the enter_result is negative something went wrong, so we just clean up and return.
            if enter_result < 0 {
                sd_bus_message_unref(reply);
                sd_bus_error_free(&mut call_error);
                sd_bus_unref(bus);
                return Err(format!("Failed to enter array container: {}", enter_result));
            }
            // else we check, whether we are at the end already. (0 = because we iterate over an array)
            let at_end = sd_bus_message_at_end(reply, 0);
            // 0 means not at end = has elements, >0 = nothing left, <0 = error - we ignore the error for now
            let has_elements = at_end == 0;
            
            // but now we know whether there are any more elements, so let's just clean up and return.
            // no need for more reading.
            sd_bus_message_exit_container(reply);
            sd_bus_message_unref(reply);
            
            has_elements
        } else {
            false
        };

        // finally, clean up the general things we don't need anymore.
        sd_bus_error_free(&mut call_error);
        sd_bus_unref(bus);
        
        Ok(has_sessions)
    }
}