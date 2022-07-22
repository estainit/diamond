#[cfg(test)]
pub mod tests_crypto {
    // use crate::lib::constants as CConsts;
    // use crate::lib::utils::cutils as cutils;
    use crate::lib::ccrypto;

    // use bech32::{self, FromBase32, ToBase32, Variant};

    #[test]
    pub fn test_bech32() {
        // let encoded = bech32::encode("bech32", vec![0x00, 0x01, 0x02].to_base32(), Variant::Bech32).unwrap();
        // assert_eq!(encoded, "bech321qqqsyrhqy2a".to_string());
        // let (hrp, data, variant) = bech32::decode(&encoded).unwrap();
        // assert_eq!(hrp, "bech32");
        // assert_eq!(Vec::<u8>::from_base32(&data).unwrap(), vec![0x00, 0x01, 0x02]);
        // assert_eq!(variant, Variant::Bech32);
        assert_eq!(ccrypto::is_valid_bech32("TP_DP"), false);
        assert_eq!(ccrypto::is_valid_bech32("im1xpjkywf48yckgepcvdnrgdrx8qurgdeevf3kyenyv9snvve5v5ung9axujl"), true);
    }


  // ecdsa tests

    #[test]
    pub fn test_autogen_ecdsa_key_pair() {
        let (status, prv1, pub1) = ccrypto::ecdsa_generate_key_pair();
        assert!(status);
        assert!(ccrypto::ecdsa_validate_private_key(&prv1));
        assert!(ccrypto::ecdsa_validate_public_key(&pub1));

        {
            let msg = "Hello world".to_string();
            let (sign_status, signature_hex, _signature) = ccrypto::ecdsa_sign_message(&prv1, &msg);
            assert!(status);
            assert!(ccrypto::ecdsa_verify_signature(&pub1, &msg, &signature_hex));
        }
    }

    /*


void TestsCCrypto::genCryptoTests()
{


  {
    QString msg = "[\"5df1b8f3e190197cd317df102be85dda74acaddb741ed5f2cbb46d73dd79e01b\"]";
    msg = CCrypto::keccak256(msg);
    msg = msg.midRef(0, CConsts::SIGN_MSG_LENGTH).toString();
    CLog::log("---------------------------------::: " + msg);
    msg = "3f060510ab5dcdf8b0f8c2a427fc96a3";
    QString signature_hex = "7538349870784dc8da9b4d624905453db99cb08072dd0c264dc82696d77f01ce134ed738a09a98d53a4d34b40f01f099309d00662a99cb23f11211a5bd20bde0";
    QString pub_key = "022968b10e02e2af51a5965b9735ac2c75c51c71207f85bec0bd49fa61902f8619";

    CLog::log("---------------------------------::signature_hex_: " + signature_hex);

    if (!CCrypto::ecdsa_verify_signature(pub_key, msg, signature_hex))
      CUtils::exiter("ECDSA JS signatured Verify failed 23", 806);
  }

  {
    QString msg = "Hello world.....................";
    QString priv_key = "e94356090b28fb09da4c03acb1f46181b094826613f5c7445cc3a5ecb5f0bf02";
    QString pub_key =  "02447246566f387d0c13912f87f5c309e3690890024131c0435d72a1443b8f2efd";
    auto[sign_status, signature_hex, signature] = CCrypto::ECDSAsignMessage(priv_key, msg);
    Q_UNUSED(signature);
    if (!sign_status)
      CUtils::exiter("ECDSA sign Message failed", 806);

    if (!CCrypto::ecdsa_verify_signature(pub_key, msg, QString::fromStdString(signature_hex)))
      CUtils::exiter("ECDSA JS signatured Verify failed 1", 806);
  }


  QString message, signature, cipher, recovered;
  message = "Hi";
  auto[isOk, prv1, pub1] = CCrypto::ECDSAGenerateKeyPair();
  if (!isOk)
  {
    CLog::log("invalid ECDSA  Generate Key Pair 1" );
    exit(213);
  }
  if (!CCrypto::ecdsa_validate_private_key(prv1))
  {
    CLog::log("invalid RSA private key auto CPP" + prv1);
    exit(213);
  }
  if (!CCrypto::ECDSAValidatePublicKey(pub1))
  {
    CLog::log("invalid RSA public key auto cpp" + pub1 );
    exit(213);
  }

  {
    QString msg = "Hello world";
    auto[sign_status, signature_hex, signature] = CCrypto::ECDSAsignMessage(prv1, msg);
    Q_UNUSED(signature);
    if (!sign_status)
        CUtils::exiter("ECDSA sign Message failed", 806);
    if (!CCrypto::ecdsa_verify_signature(pub1, msg, QString::fromStdString(signature_hex)))
    {
      CUtils::exiter("ECDSA signature Verify failed 3", 806);
    }
  }

}


  autoGenECDSAKeyPairTests();

  genCryptoTests();
//  return;

  autoGenNativeKeyPairTests();

  QString message, signature, cipher, recovered;
  bool isVerified;
  message = "Hi";





  QString prv_key_3_generatewd_by_CPP = "-----BEGIN PRIVATE KEY-----\n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\n-----END PRIVATE KEY-----";
  if (!CCrypto::isValidRSAPrivateKey(prv_key_3_generatewd_by_CPP))
  {
    CLog::log("invalid RSA private key 3 CPP" + prv_key_3_generatewd_by_CPP);
    exit(213);
  }

  QString prv_key_2_generatewd_by_JS = "-----BEGIN PRIVATE KEY-----\nMIIJQgIBADANBgkqhkiG9w0BAQEFAASCCSwwggkoAgEAAoICAQDVEb6ctK5hazO5\nVAv1lokx+VRleUZ88qT62YXFScQYsobj9C5M3/Wb8GboCQWcdmRdaEaKzIcby6OB\nIBBlCcpRawj5UYnjVieq/EApggveq8FI8ujymm83bW2vulsIve5HSKweNdKnpcYf\neZUFN3q1omFEG25Ap14V3dAxkLKh+dpqADXA8e1PzALPhf7stySt9JEaF2M9ShPe\nlnZHdnyjIA60HOvshEcM5uzXVnmx0WZlVFBXWJKDFnJLeMRiMypk3kVrFVgurrgw\ncgRgh3Aumh+ZaqQOspTCz2NOaKKuYN9oCqy7qyId0JsKmdXTbpAy1tC6SC4m/dQV\nt03+WHbY5vP04rDzTbExKC3idQsNIC+3yasmxvJZtfrrJU2SY3c4qLLfoiD19oKj\nElOPONZLkqvsldwELJq+iDWfQ1w0I4Cl17dOygc0C3/kYnykw43aMb6LnxSDj/Wa\n3PW1gSx0oWPi1sBVeA08o2XQyWZF5KMVq2LcrwfXi3C84uw3TCfJxaolOHAUJEVF\n9JXbB5jDXD6TiGawvQW9tm2XQ51B4++ptNBBNBzxJE5zcSOr/TijJzlL0PUx1oMR\n91epyqScXNze7evPDUE8Z2TDQuiLAOOnRY5JIHm7k1Ej1RjEM5Z1NoQ8akKErJyb\niSBSNgq+5yMqg8Z4uwEaYfz41FP5GwIDAQABAoICAC9tb7x6C4br0eui5baY95kE\nIuEjiakuacLcwIYV0Wyd8KC1Lck24PkBHibwOyuEvvp7x5gFxt1NCuwnMv8Kmtpc\ntPaqS8Mq3lGVLMMQi4vZN4EBcRDvh2oTFaNUHAdqH6wGZSmkWWuv3tYKvj0XCF19\n86Cxam1B1BOR+FHQH7MmZpPJ0C1OrMBibQ28ie4vJh8CnXntUs3Fu36Eh3eIzF2T\nOTKmbW2MVDRvArE8MoJ+VgigJ/IJaTN8v5xFGZGySfGB+HnJmoNpqN4/wvGptDLr\nY7VGo5YR8kMa3sQJToz6S0Bhf+mWQlXYpWpKq+oQZlCquHN+tadAQQFEeDvxfSKK\nx3mgL5DRe1/Jg4udIX3awktDTcBpJ1Mq4/Qlx1fEXY6Oq1t0AwB6OVbfbHsauDKa\nCPmzAJfTOllM6ivy1VjjTQguze88AJjQs8w3jejDuedzcrxsqAId8kQ1w3g8dl+U\nP7UiTaJEiUKnIqYQhedvuNXoyAjE13+lUPiFv7cSz8SVrvwMH4UnKaymX3bANiw+\nd421GR6Zq8GnfNX6J0ipua4M056X9ZUZH3XHPMWmOo1sq7iNJahAmaNs98oND0d3\nEpJtjdTI/pqNh5FkcesPUk+7JQjj+oJp0KZ7lf9HAJXAfSnUWzZsdiuN6Sb/68Tx\ncCUDImHFb1lt7nuBjsHhAoIBAQD2PQ9tapZaKB5NfY5P8/SU5z2VjiDP6laiM8ur\nmmwlor1+onzUeOB7KL9xzTmpuI/xIPlbWdRri+DsUwQeJxuUjEdSAACQEIUaPL5r\ndrwv8XgsXItq3uP1YicrxI48ajujQ1wml49o9Q0E/W1Ips8DIvySJ3n2wiXFS9Tp\nzDidaq60GnG4pTYHYxK1aP3mZ9C+NiO0z3R3vBJt1qBGKCUfIt4Z+2fgfofo+plC\nddyRmSjXHQDHXU6hjfMOZuMJSK21Um9Sk6VG8wMnbXliDOIQ62ZpMqLMglqJT5pI\nqyMmfR0QmJBuBwpuRrRQgwn2pXfgE48FjjSIiE6JVmTXsEYtAoIBAQDdhBF54wUe\nj7hJm6vyUe/Tprac92dqFcO5CBHlvQULaDCLHBI7HgCczVcVev0OvgjOVZtYXwoi\nB2pusGbVKai6FzHhokgV4vG+thxuCTO59eVC57lRZyHZ0I273gH/FGt8oWEZQygn\nLfXpbExsZFo2gFmmOA8sOqpRriy3WTBoALa8JFfXMUJfusEKCS9nTKqoEwh9D7He\nM1voGTht4yDShFu9nmNWNoEe449grtj8AodQBS14ZFcPBOoB8ZDUtaHAAMIDXkyc\nnuKgljpvb3+kIqrmPfTtogzwrqHU7lDfDb+8k9FNW+3KJyc8vlPziWmcxGLJ1ID9\nLKesbfnWsdFnAoIBABzqbrVanu1XaLeQ1vVsoHwb6490cUX5LtM9Yd109N42jiog\nyqfoyfOshU7H2s2jbsPSK3YexmMauf8GgKVJ0mNPVdC7T1WbmwXJvFLCTrcSlBoh\ng/24zuwx6oepPUq67uXtMxUVFzIsFKtRV1QAwq6LnT1MhXYEtorErOwHn15c+McN\nk+0q+X1ElR3YhnhYGJs9oFPLSNIGs+NSIOAcNr/BSHOkoj4qnasuQnEbhhtS8QjR\nzxWaT3GJSdGXI3IjpsJ0O75muDHst6IZqPdqq+Fww0X3PEbnx9n4DxYZgrcyWgdo\n4w0IpqDsjKAnwzPDmqAlq9XrLSAbYZ2fF5h8isECggEBAIdGXb3Hi5rMPciFzo2e\nyAzCpIZa98ZrIClg6j0YH3qF2A87x05e1hTruSCfCOb12j7XPJTNBU0epjqOc139\noNJz07xs9ASZoPPMFrSD3hjUN3uSMzOo/Z0cpVFzFFkHyVRE4R0iS+URRjmUL1+K\nMSVTYGwHlAx4jwZujVQynUMGHJHsx8/cfyIi1Dwiu9YjlZwqc2acLQz65EdrD3Db\n4XFt7vBvR06g3l1Urnxmio/ro2KUpt38uMWtRJKHSOD9QpgUXmbcMYD77n4PAASJ\nXLOJkEAm2eo8qhZZLu7v0mdE6DBrPMg3OZHYMzf/KxlwcxSPQZzrrxHtMUA1sPsR\ngEUCggEAHP+ELenrAP3TpEB1D59Dd8pBtKrVMQWvsVj5zjEe5qttC6ECkII0PoFK\nZqD1xzWsFJnrWQTC+qowmRXD+DsAshxYbc2wXk9nezchKGU3BACqiMqhRZEzJN4e\nyZ2B/qdlaAi7j6ZJXd3s+OHtrWeJw1VJtaOBYQK8TWcZhHNbyb86+hzpssfBhTdg\n0s2Ai4tzocm70IKrSmDq76IvhwRO9A4l7Im1fiqB5E+gvhDaSfhwM+j42NxELE21\nMDgfKw7uhzmmcDFixxJ32TtiIC6K8F3SpeTUkTOuLNESd4y9idINhdfVKneNe++Z\nPFrnBwb6E/LcKjzkUat3PUFREuhAKQ==\n-----END PRIVATE KEY-----\n";
  if (!CCrypto::isValidRSAPrivateKey(prv_key_2_generatewd_by_JS))
  {
    CLog::log("invalid RSA private key 2" + prv_key_2_generatewd_by_JS );
    exit(213);
  }





  QString pub_key_2_generatewd_by_JS = "-----BEGIN PUBLIC KEY-----\nMIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA1RG+nLSuYWszuVQL9ZaJ\nMflUZXlGfPKk+tmFxUnEGLKG4/QuTN/1m/Bm6AkFnHZkXWhGisyHG8ujgSAQZQnK\nUWsI+VGJ41YnqvxAKYIL3qvBSPLo8ppvN21tr7pbCL3uR0isHjXSp6XGH3mVBTd6\ntaJhRBtuQKdeFd3QMZCyofnaagA1wPHtT8wCz4X+7LckrfSRGhdjPUoT3pZ2R3Z8\noyAOtBzr7IRHDObs11Z5sdFmZVRQV1iSgxZyS3jEYjMqZN5FaxVYLq64MHIEYIdw\nLpofmWqkDrKUws9jTmiirmDfaAqsu6siHdCbCpnV026QMtbQukguJv3UFbdN/lh2\n2Obz9OKw802xMSgt4nULDSAvt8mrJsbyWbX66yVNkmN3OKiy36Ig9faCoxJTjzjW\nS5Kr7JXcBCyavog1n0NcNCOApde3TsoHNAt/5GJ8pMON2jG+i58Ug4/1mtz1tYEs\ndKFj4tbAVXgNPKNl0MlmReSjFati3K8H14twvOLsN0wnycWqJThwFCRFRfSV2weY\nw1w+k4hmsL0FvbZtl0OdQePvqbTQQTQc8SROc3Ejq/04oyc5S9D1MdaDEfdXqcqk\nnFzc3u3rzw1BPGdkw0LoiwDjp0WOSSB5u5NRI9UYxDOWdTaEPGpChKycm4kgUjYK\nvucjKoPGeLsBGmH8+NRT+RsCAwEAAQ==\n-----END PUBLIC KEY-----\n";
  if (!CCrypto::isValidRSAPublicKey(pub_key_2_generatewd_by_JS))
  {
    CLog::log("invalid RSA public key 2" + pub_key_2_generatewd_by_JS );
    exit(213);
  }


  QString pub_key_1 = "-----BEGIN PUBLIC KEY-----\nMIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA1RG+nLSuYWszuVQL9ZaJ\nMflUZXlGfPKk+tmFxUnEGLKG4/QuTN/1m/Bm6AkFnHZkXWhGisyHG8ujgSAQZQnK\nUWsI+VGJ41YnqvxAKYIL3qvBSPLo8ppvN21tr7pbCL3uR0isHjXSp6XGH3mVBTd6\ntaJhRBtuQKdeFd3QMZCyofnaagA1wPHtT8wCz4X+7LckrfSRGhdjPUoT3pZ2R3Z8\noyAOtBzr7IRHDObs11Z5sdFmZVRQV1iSgxZyS3jEYjMqZN5FaxVYLq64MHIEYIdw\nLpofmWqkDrKUws9jTmiirmDfaAqsu6siHdCbCpnV026QMtbQukguJv3UFbdN/lh2\n2Obz9OKw802xMSgt4nULDSAvt8mrJsbyWbX66yVNkmN3OKiy36Ig9faCoxJTjzjW\nS5Kr7JXcBCyavog1n0NcNCOApde3TsoHNAt/5GJ8pMON2jG+i58Ug4/1mtz1tYEs\ndKFj4tbAVXgNPKNl0MlmReSjFati3K8H14twvOLsN0wnycWqJThwFCRFRfSV2weY\nw1w+k4hmsL0FvbZtl0OdQePvqbTQQTQc8SROc3Ejq/04oyc5S9D1MdaDEfdXqcqk\nnFzc3u3rzw1BPGdkw0LoiwDjp0WOSSB5u5NRI9UYxDOWdTaEPGpChKycm4kgUjYK\nvucjKoPGeLsBGmH8+NRT+RsCAwEAAQ==\n-----END PUBLIC KEY-----\n";
  QString prv_key_1 = "-----BEGIN PRIVATE KEY-----\n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\nPFrnBwb6E/LcKjzkUat3PUFREuhAKQ==\n-----END PRIVATE KEY-----\n";
  QString message_1 = "Hi";


  // test 1 encryption
  cipher = CCrypto::encryptStringWithPublicKey(
    pub_key_1,
    message_1
  );
  CLog::log("encryptStringWithPublicKey cipher " + cipher );
  QString clearText = CCrypto::decryptStringWithPrivateKey(
    prv_key_1,
    cipher
  );
  if (clearText != message_1)
  {
    CLog::log("failed decryptStringWithPrivateKey1 " + clearText );
    exit(213);
  }

  clearText = CCrypto::decryptStringWithPrivateKey(
    prv_key_1,
    "V5Bt0DgdNqaiZAR5hhJ6bz/RD6fO+4lzy0mkdsmwBX4rgiidQtl2bFtM8l4UV9/ml8wmbVQGCUHdzK/SR+goiG8x+TD7nzf3vi2WRgOsS90KJWAdKKlaPcj++pMm3cFzMw4XOsB/tqM7IiulVG2OzRc3ct2bubs7ZKwEMc6OALqIDyOXZC3AJrIjpiJ10MHWtOGkAcouQKsdZfjbDSKiTVynywrJVyYqZTj2LO+8HIwVxjrJB/087euCtPeQR3XmaYQRDnkjMsIaIG0RzeRWgow456r01F+O0Tp2a1WYeyrUFmOL+6Bo9bFzDSlEJpZEi15oVb2/dnBwLztu/bpy3q6CFK8uWzg/70xGrqXVvUCGjOF7MtTJ3T4qxBz1zEXhbRkmDVcZS3WMSp80mndXDX9aUGdXy1pRHlKJpB0IMwLT7TGXAeQKEaq5u/LTfdHxcsyGxVntNchwcSAivmidirNQqRefpWLByLTQedczyZrlORyieBaw9d+gH3XjVKu0Lut5Lkame6Gjfp0HfoE/oCtb+cr8olFNSHY/+GJ/lKM1ZCbR+weByFXDKkkvoDiHQFXQY3vM1rfrDR7oxjjE+Z8oYP7ST+YM4sDvuw8tqYEcv2yBXF+GfJsA9hJAXKJM4g+5i2j7CBcB/aKg5UsU+8AcwpL7wUN0q5MvOMCU4tI="
  );
  if (clearText != message_1)
  {
    CLog::log("failed decryptStringWithPrivateKey JS encrypted cipher " + clearText );
    exit(213);
  }






  // test signature & verification
  signature = CCrypto::nativeSign(
    prv_key_1,
    message_1
  );
  CLog::log("signature " + signature );
  if (signature != "KdFV+yVbsom2G4yLxiaeN7mmKChnYhMeZgjMbUh6QHYmnWRa0NkirBXTOnpSDqPbZ1bTO29J0A+CRqUYGCr0VLSCFp/eKCtlx54Ipr0y2EsWz/LtynbiiGFx4WbdJFOv4tbctjimypDslxOZLfspqVNHgpt11KFji2wR+RY13mpkAmRsbqz0+narkAYRTTuS6Trg4qzRLPoPtXM25Q9GuvvKTnwR/ZFnRBbufqQ9YuTLY15sA3tmlhPapMgAnR8yE4Hw6rc1xDcQwYWUzfW2RbWZncOL4km6u7TX3VzBxdzrg1eheGi8WC5BYLvu6djeGMiNEz8SdkPinppwn6LXUQwSwOXQCd6y1u26XthP+L78qwXkTx+mpr6kAzfQz2mHWmQ5sUOE+xPrkmNy9Qa667jRpjQkO7L42IxUVD7jYm4l4AYSEoV2cTYCU0/TbnLBv+grsZqkKLv+NFomvrPvt1bVzG1nJFyOQQAbvKYkxHoKmQ9uIzPdsqXu34Sanp2yyYJa44JIApP2j9rqcUD5U5TOXWvANXHw7igmkqE+EGtQ441q24odBcwTkDkfOcxBXTA9dTHhgnQHgR43czJv4yGKpH9srXTOAFYghh+Bc8pA+YfT9fAsRX2c6Z+ZOkm4pLrrbgc7o110peqATojlcSz/oXHoyhIYR/egXtaeNWc=")
  {
    CLog::log("failed nativeSign1 " + signature );
    exit(213);
  }
  isVerified = CCrypto::nativeVerifySignature(pub_key_1, message_1, signature);
  if (!isVerified)
  {
    CLog::log("failed nativeVerifySignature1");
    exit(213);
  }






  // test 2 encryption
  QString res = CCrypto::encryptStringWithPublicKey(
    "-----BEGIN PUBLIC KEY-----\nMIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA1RG+nLSuYWszuVQL9ZaJ\nMflUZXlGfPKk+tmFxUnEGLKG4/QuTN/1m/Bm6AkFnHZkXWhGisyHG8ujgSAQZQnK\nUWsI+VGJ41YnqvxAKYIL3qvBSPLo8ppvN21tr7pbCL3uR0isHjXSp6XGH3mVBTd6\ntaJhRBtuQKdeFd3QMZCyofnaagA1wPHtT8wCz4X+7LckrfSRGhdjPUoT3pZ2R3Z8\noyAOtBzr7IRHDObs11Z5sdFmZVRQV1iSgxZyS3jEYjMqZN5FaxVYLq64MHIEYIdw\nLpofmWqkDrKUws9jTmiirmDfaAqsu6siHdCbCpnV026QMtbQukguJv3UFbdN/lh2\n2Obz9OKw802xMSgt4nULDSAvt8mrJsbyWbX66yVNkmN3OKiy36Ig9faCoxJTjzjW\nS5Kr7JXcBCyavog1n0NcNCOApde3TsoHNAt/5GJ8pMON2jG+i58Ug4/1mtz1tYEs\ndKFj4tbAVXgNPKNl0MlmReSjFati3K8H14twvOLsN0wnycWqJThwFCRFRfSV2weY\nw1w+k4hmsL0FvbZtl0OdQePvqbTQQTQc8SROc3Ejq/04oyc5S9D1MdaDEfdXqcqk\nnFzc3u3rzw1BPGdkw0LoiwDjp0WOSSB5u5NRI9UYxDOWdTaEPGpChKycm4kgUjYK\nvucjKoPGeLsBGmH8+NRT+RsCAwEAAQ==\n-----END PUBLIC KEY-----\n",
    "Hi"
  );


  // test sha256
  QString hash_sha256 = CCrypto::sha256("imagine");
  if (hash_sha256 != "7fdd65bbaaa687d48dc85a26a6dd7ef17fa379994fd8f3b26e19fe36976aeadf")
    CUtils::exiter("Failed in sha256 imagine!", 01);

}



bool TestsCCrypto::test_b64()
{
  QString clear_text = "hello world";
  QString b64 = CCrypto::base64Encode(clear_text);
  if (CCrypto::base64Decode(b64) != clear_text)
    CUtils::exiter("base 64 Encode/Decode Failed1", 0);

  return true;
}






*/
    #[test]
    pub fn test_autogen_native_key_pairs()
    {
        let message: String = "".to_string();
        let signature: String = "".to_string();
        let cipher: String = "".to_string();
        let recovered: String = "".to_string();
        let isVerified: bool = false;
        let message: String = "Hi".to_string();
        let (isOk, prv1, pub1) = ccrypto::native_generate_key_pair();
        assert!(isOk);

        /*
          if (!CCrypto::isValidRSAPrivateKey(prv1))
          {
            CLog::log("invalid RSA private key auto CPP" + prv1);
            exit(213);
          }
          if (!CCrypto::isValidRSAPublicKey(pub1))
          {
            CLog::log("invalid RSA public key auto cpp" + pub1 );
            exit(213);
          }

          // test signature & verification
          signature = CCrypto::nativeSign(
            prv1,
            message
          );
          isVerified = CCrypto::nativeVerifySignature(pub1, message, signature);
          if (!isVerified)
          {
            CLog::log("failed nativeVerifySignature auto1");
            exit(213);
          }

          // test auto encryption Pub -> Priv
          cipher = CCrypto::encryptStringWithPublicKey(
            pub1,
            message
          );
          recovered = CCrypto::decryptStringWithPrivateKey(
            prv1,
            cipher
          );
          if (recovered != message)
          {
            CLog::log("failed decryptStringWithPrivateKey auto 1 " + recovered );
            exit(213);
          }
        */
    }
}