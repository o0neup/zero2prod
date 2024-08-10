use super::subscriber_name::SubscriberName;

#[derive(Debug)]
pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}
