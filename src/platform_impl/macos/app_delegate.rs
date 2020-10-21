use super::{activation_hack, app_state::AppState};
use cocoa::base::id;
use objc::{
    declare::ClassDecl,
    runtime::{Class, Object, Sel},
};
use std::os::raw::c_void;
use crate::platform_impl::platform::event::EventWrapper;
use crate::event::Event::OpenFilesEvent;
use std::path::Path;

pub struct AppDelegateClass(pub *const Class);
unsafe impl Send for AppDelegateClass {}
unsafe impl Sync for AppDelegateClass {}

lazy_static! {
    pub static ref APP_DELEGATE_CLASS: AppDelegateClass = unsafe {
        let superclass = class!(NSResponder);
        let mut decl = ClassDecl::new("WinitAppDelegate", superclass).unwrap();

        decl.add_class_method(sel!(new), new as extern "C" fn(&Class, Sel) -> id);
        decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));
        decl.add_method(
            sel!(applicationDidFinishLaunching:),
            did_finish_launching as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(applicationDidBecomeActive:),
            did_become_active as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(applicationDidResignActive:),
            did_resign_active as extern "C" fn(&Object, Sel, id),
        );

        decl.add_ivar::<*mut c_void>(activation_hack::State::name());
        decl.add_method(
            sel!(activationHackMouseMoved:),
            activation_hack::mouse_moved as extern "C" fn(&Object, Sel, id),
        );

        decl.add_method(
            sel!(application:openFiles:),
            open_files as extern "C" fn(&Object, Sel, id, id)
        );

        decl.add_method(
                    sel!(application:openUrls:),
                    open_urls as extern "C" fn(&Object, Sel, id, id)
        );


        AppDelegateClass(decl.register())
    };
}

extern "C" fn new(class: &Class, _: Sel) -> id {
    unsafe {
        let this: id = msg_send![class, alloc];
        let this: id = msg_send![this, init];
        (*this).set_ivar(
            activation_hack::State::name(),
            activation_hack::State::new(),
        );
        this
    }
}

extern "C" fn dealloc(this: &Object, _: Sel) {
    unsafe {
        activation_hack::State::free(activation_hack::State::get_ptr(this));
    }
}

extern "C" fn did_finish_launching(_: &Object, _: Sel, _: id) {
    trace!("Triggered `applicationDidFinishLaunching`");
    AppState::launched();
    trace!("Completed `applicationDidFinishLaunching`");
}

extern "C" fn did_become_active(this: &Object, _: Sel, _: id) {
    trace!("Triggered `applicationDidBecomeActive`");
    unsafe {
        activation_hack::State::set_activated(this, true);
    }
    trace!("Completed `applicationDidBecomeActive`");
}

extern "C" fn did_resign_active(this: &Object, _: Sel, _: id) {
    trace!("Triggered `applicationDidResignActive`");
    unsafe {
        activation_hack::refocus(this);
    }
    trace!("Completed `applicationDidResignActive`");
}

extern "C" fn open_files(_this: &Object, _: Sel, app: id, files: id) {
    trace!("Triggered `application:openFiles:`");
    use cocoa::foundation::NSFastEnumeration;

    let mut paths: Vec<String> = Vec::new();
    for file in unsafe { files.iter() } {
        use cocoa::foundation::NSString;
        use std::ffi::CStr;
        unsafe {
            let utf8string = NSString::UTF8String(file);
            let str = CStr::from_ptr(utf8string).to_string_lossy().into_owned();
            if Path::new(&str).exists() {
                paths.push(str)
            }
        }
    }
    if paths.is_empty() {
        let _: () = unsafe { msg_send![app, replyToOpenOrPrint:1] };
    } else {
        let _: () = unsafe { msg_send![app, replyToOpenOrPrint:0] };
        AppState::queue_event(EventWrapper::StaticEvent(OpenFilesEvent(paths)));
    }
    trace!("Completed `application:openFiles:`");
}

extern "C" fn open_urls(_this: &Object, _: Sel, app: id, urls: id) {
    trace!("Triggered `application:openUrls:`");
    use cocoa::foundation::NSFastEnumeration;

    let mut url_strings: Vec<String> = Vec::new();
    for url in unsafe { urls.iter() } {
        use cocoa::foundation::NSString;
        use std::ffi::CStr;
        unsafe {
            let url_string: id = msg_send![url, absoluteString];
            let utf8string = NSString::UTF8String(url_string);
            let str = CStr::from_ptr(utf8string).to_string_lossy().into_owned();
            url_strings.push(str);
        }
    }
    AppState::queue_event(EventWrapper::StaticEvent(OpenFilesEvent(url_strings)));
    trace!("Completed `application:openUrls:`");
}