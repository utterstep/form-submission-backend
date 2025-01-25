use eyre::{Result, WrapErr};
use lettre::{
    message::{header::ContentType, Mailbox},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

#[tracing::instrument(skip(mailer, body), err)]
pub async fn send_mail(
    mailer: &AsyncSmtpTransport<Tokio1Executor>,
    to: &str,
    from: &str,
    subject: &str,
    body: &str,
) -> Result<()> {
    let from: Mailbox = from.parse().wrap_err("Failed to parse from address")?;
    let reply_to = from.clone();
    let to = to.parse().wrap_err("Failed to parse to address")?;

    let email = Message::builder()
        .from(from)
        .reply_to(reply_to)
        .to(to)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body.to_string())
        .wrap_err("Failed to build email")?;

    match mailer.send(email).await {
        Ok(_response) => Ok(()),
        Err(e) => {
            tracing::error!("Error from lettre: {:?}", e);

            Err(e).wrap_err("Failed to send email")
        }
    }
}
