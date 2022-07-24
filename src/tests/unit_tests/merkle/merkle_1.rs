
#[cfg(test)]
pub mod merkel_tests1 {
    // use crate::lib::utils::cutils;

    #[test]
    pub fn test_do_panic() {
        assert!(1==1);

    }
}


/*

void CMerkleTests1::doTests()
{
  {
    auto[root, proofs, version, levels, leaves] = CMerkle::generate({"a"}, "hashed", "noHashed");
    Q_UNUSED(levels);
    Q_UNUSED(leaves);
    Q_UNUSED(version);
    if (root != "a")
    {
      CLog::log("ERROR in CMerkle::generate 1: " , "app", "fatal");
      exit(1098);
    }
    if (proofs.size() != 0)
    {
      CLog::log("ERROR in CMerkle::generate 1: " , "app", "fatal");
      exit(1098);
    }
  }

  {
    auto[root, proofs, version, levels, leaves] = CMerkle::generate({"1"}, "string");
    Q_UNUSED(levels);
    Q_UNUSED(leaves);
    Q_UNUSED(version);
    if (root != CCrypto::keccak256("1"))
    {
      CLog::log("ERROR in CMerkle::generate 1.1: " , "app", "fatal");
      exit(1098);
    }
    if (proofs.size() != 0)
    {
      CLog::log("ERROR in CMerkle::generate 1.1: " , "app", "fatal");
      exit(1098);
    }
  }

}



 */