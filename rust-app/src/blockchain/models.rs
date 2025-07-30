use alloy::sol;
use serde::Serialize;

sol! {
    #[sol(rpc)]
    contract OpenxAIGenesis {
          event Participated(
            uint256 indexed tier,
            address indexed account,
            uint256 amount
        );
    }

    #[sol(rpc)]
    contract OpenxAIClaimer {
        event TokensClaimed(address indexed account, uint256 total, uint256 released);
    }

    #[derive(Serialize)]
    struct Claim {
        address claimer;
        uint256 total;
    }
}
