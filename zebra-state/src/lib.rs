#![doc(html_logo_url = "https://www.zfnd.org/images/zebra-icon.png")]
#![doc(html_root_url = "https://doc.zebra.zfnd.org/zebra_storage")]
use futures::prelude::*;
use std::{
    collections::HashMap,
    error::Error,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{buffer::Buffer, Service};
use zebra_chain::block::{Block, BlockHeaderHash};

#[derive(Debug)]
pub enum Request {
    // TODO(jlusby): deprecate in the future based on our validation story
    AddBlock { block: Arc<Block> },
    GetBlock { hash: BlockHeaderHash },
}

#[derive(Debug)]
pub enum Response {
    Added,
    Block { block: Arc<Block> },
}

pub mod in_memory {
    use super::*;
    use std::error::Error;

    pub fn init() -> impl Service<
        Request,
        Response = Response,
        Error = Box<dyn Error + Send + Sync + 'static>,
        Future = impl Future<Output = Result<Response, Box<dyn Error + Send + Sync + 'static>>>,
    > + Send
           + Clone
           + 'static {
        Buffer::new(ZebraState::default(), 1)
    }
}

#[derive(Default)]
struct ZebraState {
    blocks: HashMap<BlockHeaderHash, Arc<Block>>,
}

impl Service<Request> for ZebraState {
    type Response = Response;
    type Error = Box<dyn Error + Send + Sync + 'static>;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        match req {
            Request::AddBlock { block } => {
                let hash = block.as_ref().into();
                self.blocks.insert(hash, block);

                async { Ok(Response::Added) }.boxed()
            }
            Request::GetBlock { hash } => {
                let result = self
                    .blocks
                    .get(&hash)
                    .cloned()
                    .map(|block| Response::Block { block })
                    .ok_or_else(|| "block could not be found".into());

                async move { result }.boxed()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Report;
    use eyre::{bail, ensure, eyre};
    use hex::FromHex;
    use zebra_chain::serialization::ZcashDeserialize;

    lazy_static::lazy_static! {
        pub static ref BLOCK_MAINNET_415000_BYTES: Vec<u8> = <Vec<u8>>::from_hex("040000005274b43b9e4ad8f43e93f78463d24dcfe531aeb4719819f4f97f7e0300000000663073bc4bfa95c9bec36aad7268a573049797bdfc5aa4c743fbe4820aa393ce0000000000000000000000000000000000000000000000000000000000000000a8becc5be1ab031cc2fd607c776a7a0000000000000000000000000000000000000000003eb21819fd400500949d55de0cc633e0cce41e4649ef4aa3349f0100290ffe281b947b3b53fbd2f35b1ce292649b96ac6e0883af3a6844b95592e74556da344b4701961cd4130c68219cfa1341d5afb5049eb0e8be4a2d92d678c40785e33705548b5f3a54f0a4c39a2f58ee784a24163cd86f54812327df55e1d55ca84b6e7b887a7cbfb9091a585bdb8ea4759307c56c1b3dafc669245a6f654b6f730052266a01ad4f9c0b59ed4e17712b3e72df0498aa8de4888f993531c60acded1d4b66e89de0b6482cccd4a712f5cf9d4ca83be0f922de2c1dbb3a1407480dbe8795993d8be640988abfe7a8a1b33a12131c451e1abc0d83fb851862c637ce724d5fe97aa9a806cf34bab509f4554b0cd10a7ddfd5821b091ad2c90c1aa1d81eb3d72db41993b648f41e2138ff9531a30ff73b22140e4ebd7baa33848e512d99300c5c131c6e75f5714a5c6dcb178b4a4978dac83ad412fbd692019250c553049aad457984bedfc96ae701c659bc7007a97d0a9002b945bdec45a945ef6285b2cd553b4c09d907c627863f0399e8725b4ff7fc5979e3cff22814508448ef8b9831c285959333396aa362a51cf205097afabec15e41fb6e30b622374bf58b37ef9d1b241ead5a682b98b65749a57568e238d50afd417e1e960e7b5a064fd9f694d783a2cbcd58552dedbb9e5e1123674ef73a524196cf05d3e524660549ffe7bd6568057135ffd5afd943f6da11cbb597e8ccecd77ecbe909de0631bfa29cd3e3d5544671ba80256153d6e9990b88ad8e0cf4989bef4be457f9c7b0f1aacd6e0ef320605c29ed0cd2eb6cfce216c52a317580201cad7a0943d24b7b06d5bf758761dd96e11970b5ded697222b2c77e7f256a605ac755549c1651f25adfc9d53d9117e3a0bb409eee4a600120472949c7dda1c2edb3c330c7f96179982916457d331e96309dd24df74eedd00e7db497ee130f77de666eb557fb316e87adaf1813ce426a458a6eee3a85b2ab88f6553aadae8de652e211a1d9f334d596b5eb6173407efcc2e8154bb9ca1212aa9a1a1121d2f5a7712cf25cc8148b8052e0d2e09f20e5ba2a98277e975b0eed9a89206966337163f215c9d04a6598b0958d333d846773c69e5abfd0a0427f3660614dd82b79adb851a0d58b62df5f0b3ac836e6e25f3a51f49a99ade57796fe9fcc26f0a1f94ff0819fe52b75087edbed3a81626eb5416c66557f11c0fcedff223d6aa8cd5c35386e5b4b95a0f0392ca301a38b3687d094493b9e9d264d07a190ce57d116804382a3fabe15af4df4fa043f0287aa1ed5568d9ef5d12510d010ccdab4eb616f6df13bb3126ef43d9d65735e4e4c04b576348d040b535055a3d5ae191b75f0612f3b24066a05245f27fe57bda66bd6dec7e4fc9cb236802062adde3cd0e313482c92a0c721102b1f38b015ab8d01559cbcb40f674e9efad5ee9c2fe133faa55ca1dd0ff26710f9da819cc1459cb7ed260dad3db0596258d47c74c32a8b852b671c5a0caa2001603d90c91a7df2e2d4ee9ae9bf1a6b1ec88151c62360d03024d2e2d0114084f6b88c5bba24aa7cecfac16e91e0baf3d8653e218093e81d2a63c32eff1d9030f9e1414ece420daa24e0dd5b845b3274bb839ca1c53bcc0194242d74b2631b9495a654fbbdcbfad779f7322b60736249880604821d96924e3fa397f354a5ecca34f614da5456f9b36338c37d8f6fbf626be983477766022872746da10a1771ceb02dd8aac01ba186bf1488630479e1284da0190fce8b59ac6b0fd416bee56b72f0a5845153557ff0f4950a0dc5be65ce942d22e18534c4e0efabb2d1525dc4858b9b0f77d474a125ebc250e08fedbfaa66f453d90932cab3ff45221909968e51e6bc254d509adeb75cba76d48fe024e3e66d8df5e01030000807082c403010000000000000000000000000000000000000000000000000000000000000000ffffffff1a03185506152f5669614254432f48656c6c6f20776f726c64212fffffffff0200ca9a3b000000001976a914fb8a6a4c11cb216ce21f9f371dfc9271a469bd6d88ac80b2e60e0000000017a914e0a5ea1340cc6b1d6a82c06c0a9c60b9898b6ae987000000000000000000").expect("Block bytes are in valid hex representation");
    }

    #[tokio::test]
    async fn round_trip() -> Result<(), Report> {
        let block: Arc<_> = Block::zcash_deserialize(&BLOCK_MAINNET_415000_BYTES[..])?.into();
        let hash = block.as_ref().into();

        let mut service = in_memory::init();

        let response = service
            .call(Request::AddBlock {
                block: block.clone(),
            })
            .await
            .map_err(|e| eyre!(e))?;

        ensure!(
            matches!(response, Response::Added),
            "unexpected response kind: {:?}",
            response
        );

        let block_response = service
            .call(Request::GetBlock { hash })
            .await
            .map_err(|e| eyre!(e))?;

        match block_response {
            Response::Block {
                block: returned_block,
            } => assert_eq!(block, returned_block),
            _ => bail!("unexpected response kind: {:?}", block_response),
        }

        Ok(())
    }
}
