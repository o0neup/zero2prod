use super::{subscriber_email::SubscriberEmail, subscriber_name::SubscriberName};

#[derive(Debug)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
