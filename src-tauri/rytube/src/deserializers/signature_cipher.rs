use serde::{Deserialize, de::Error, de::Unexpected};
use crate::structs::stream_elements::SignatureCipher;


#[derive(Debug, Deserialize)]
struct SignatureParts{
    url: String,
    s: String
}

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<SignatureCipher, D::Error>
    where
        D: serde_with::serde::Deserializer<'de> {


    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct EitherUrlOrCipher {
        url: Option<String>,
        #[serde(default)]
        #[serde(alias = "Cipher")]
        #[serde(deserialize_with = "parse_signature")]
        signature_cipher: Option<SignatureParts>,
    }
    
    let both: EitherUrlOrCipher = Deserialize::deserialize(deserializer)?;
    match (both.url, both.signature_cipher) {
        (Some(url), None) => Ok(SignatureCipher::Url(url)),
        (None, Some(signature)) => Ok(SignatureCipher::Signature { url: signature.url, s: signature.s }),
        (None, None) => Err(serde_with::serde::de::Error::missing_field("signatureCipher")),
        (Some(_), Some(_)) => Err(serde_with::serde::de::Error::duplicate_field("url")),
    }
}

fn parse_signature<'de, D>(deserializer: D) -> Result<Option<SignatureParts>, D::Error>
where
D: serde_with::serde::Deserializer<'de>{
    let signature = String::deserialize(deserializer)?;
    serde_qs::from_str::<SignatureParts>(signature.as_str())
        .map(Some)
        .map_err(|_| D::Error::invalid_value(
            Unexpected::Str(signature.as_str()),
            &"a valid SignatureCipher",
        ))
}