use super::{MimeType, VideoQuality, QualityLabel, AudioQuality, SignatureCipher};
use serde::{Deserialize};
use serde_with::json::JsonString;
use serde_with::serde_as;

#[serde_as]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RawStream{
    pub itag: u64,
    #[serde(with = "crate::deserializers::mime_type")]
    pub mime_type: MimeType,
    pub bitrate: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde_as(as = "Option<JsonString>")]
    pub content_length: Option<u64>,
    pub quality: Option<VideoQuality>,
    pub fps: Option<u16>,
    pub quality_label: Option<QualityLabel>,
    pub average_bitrate: Option<u64>,
    pub audio_quality: Option<AudioQuality>,
    #[serde_as(as = "Option<JsonString>")]
    pub audio_sample_rate: Option<u32>,
    pub audio_channels: Option<u8>,
    #[serde(flatten, with = "crate::deserializers::signature_cipher")]
    pub signature_cipher: SignatureCipher
}


/*
{
          "itag": 18,
          "mimeType": "video/mp4; codecs=\"avc1.42001E, mp4a.40.2\"",
          "bitrate": 554841,
          "width": 640,
          "height": 360,
          "lastModified": "1609509914412265",
          "contentLength": "15888937",
          "quality": "medium",
          "fps": 30,
          "qualityLabel": "360p",
          "averageBitrate": 554636,
          "audioQuality": "AUDIO_QUALITY_LOW",
          "approxDurationMs": "229180",
          "audioSampleRate": "44100",
          "audioChannels": 2,
          "signatureCipher": "s=j47g47fKdqA7c-ERZ5beVjI4g58wcUCL9STMZHz0OKg0efhAEiA%3Di92OAIh-eebqODxwMtqHmb8FHA-EAE-XT3xlsD1TPLAhIgRw8JQ0qOA&sp=sig&url=https://rr5---sn-ug5onuxaxjvh-n8ml.googlevideo.com/videoplayback%3Fexpire%3D1659315458%26ei%3DotDmYsCjLfKNv_IP1LW3sA0%26ip%3D94.190.37.241%26id%3Do-AEXkCi9_hg7f3yg4icNrYLil616uXfyrolrWoDN2hkqc%26itag%3D18%26source%3Dyoutube%26requiressl%3Dyes%26mh%3Dex%26mm%3D31%252C29%26mn%3Dsn-ug5onuxaxjvh-n8ml%252Csn-ug5onuxaxjvh-n8vs%26ms%3Dau%252Crdu%26mv%3Dm%26mvi%3D5%26pl%3D22%26initcwndbps%3D1098750%26spc%3DlT-KhvglMxAIfOOdu93y_21FvjBH0oE%26vprv%3D1%26mime%3Dvideo%252Fmp4%26ns%3DdSHLhMHMZDQ7WK9ypFjxHvUH%26gir%3Dyes%26clen%3D15888937%26ratebypass%3Dyes%26dur%3D229.180%26lmt%3D1609509914412265%26mt%3D1659293576%26fvip%3D2%26fexp%3D24001373%252C24007246%26c%3DWEB%26rbqsm%3Dfr%26txp%3D5432434%26n%3DSGatxQN3WupKaeXX%26sparams%3Dexpire%252Cei%252Cip%252Cid%252Citag%252Csource%252Crequiressl%252Cspc%252Cvprv%252Cmime%252Cns%252Cgir%252Cclen%252Cratebypass%252Cdur%252Clmt%26lsparams%3Dmh%252Cmm%252Cmn%252Cms%252Cmv%252Cmvi%252Cpl%252Cinitcwndbps%26lsig%3DAG3C_xAwRAIgDT_LGJ5S3E_B0w0HfsvEo8aZHGRtYlghRzBbl9a2X04CIGELBqRqPq6wb6hkNsoI2OZP-OW3iUAS6q0mc0808aoh"
        }
*/