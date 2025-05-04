use base64ct::{Base64, Encoding};
use clap::Parser;
use objc2::{
    AnyThread, DefinedClass, define_class, msg_send,
    rc::Retained,
    runtime::{NSObject, NSObjectProtocol},
    sel,
};
use objc2_app_kit::{
    NSBitmapImageFileType, NSBitmapImageRep, NSRunningApplication, NSWorkspace,
    NSWorkspaceApplicationKey, NSWorkspaceDidActivateApplicationNotification,
};
use objc2_foundation::{NSDictionary, NSNotification, NSRunLoop};
use serde_json::json;
use std::ptr::null_mut;
use tracing::{error, info};
use ureq::http::header;

#[derive(Debug, Parser)]
struct Arguments {
    #[arg(long)]
    token: String,
    #[arg(long, default_value = "https://makatime.kallydev.workers.dev")]
    endpoint: String,
}

struct ReporterIvars {
    endpoint: String,
    token: String,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = ReporterIvars]
    struct Reporter;

    unsafe impl NSObjectProtocol for Reporter {}

    impl Reporter {
        #[unsafe(method(applicationDidActivate:))]
        fn application_did_activate(&self, notification: &NSNotification) {
            let authorization_header = (header::AUTHORIZATION, format!("Bearer {}", self.ivars().token));

            unsafe {
                let user_info = notification.userInfo().unwrap();
                let application = user_info.valueForKey(NSWorkspaceApplicationKey).unwrap();
                let application = application.downcast_ref::<NSRunningApplication>().unwrap();

                let application_name = application.localizedName().unwrap();

                let application_icon = application.icon().unwrap();
                let application_icon = application_icon.CGImageForProposedRect_context_hints(null_mut(), None, None).unwrap();
                let application_icon = NSBitmapImageRep::initWithCGImage(NSBitmapImageRep::alloc(), &application_icon);
                let application_icon = application_icon.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new()).unwrap();

                match application_name.to_string().as_str() {
                    "loginwindow" => {
                        let _ = ureq::delete(&self.ivars().endpoint)
                            .header(authorization_header.0, authorization_header.1)
                            .call()
                            .inspect_err(|error| error!(?error, "update activity"));

                        info!("deleted current activity")
                    },
                    application_name => {
                        let activity = json!({
                            "icon": Some(format!("data:image/png;base64,{}", Base64::encode_string(application_icon.as_bytes_unchecked()))),
                            "name": application_name,
                        });

                        let _ = ureq::put(&self.ivars().endpoint)
                            .header(authorization_header.0, authorization_header.1)
                            .send_json(activity)
                            .inspect_err(|error| error!(?error, "delete activity"));

                        info!(application_name, "updated current activity")
                    },
                }
            }
        }
    }
);

impl Reporter {
    pub fn new(endpoint: &str, token: &str) -> Retained<Self> {
        let this = Self::alloc().set_ivars(ReporterIvars {
            endpoint: endpoint.to_owned(),
            token: token.to_owned(),
        });

        unsafe { msg_send![super(this), init] }
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let arguments = Arguments::try_parse()?;

    unsafe {
        let reporter = Reporter::new(&arguments.endpoint, &arguments.token);

        NSWorkspace::sharedWorkspace()
            .notificationCenter()
            .addObserver_selector_name_object(
                &reporter,
                sel!(applicationDidActivate:),
                Some(NSWorkspaceDidActivateApplicationNotification),
                None,
            );

        NSRunLoop::mainRunLoop().run();
    }

    Ok(())
}
