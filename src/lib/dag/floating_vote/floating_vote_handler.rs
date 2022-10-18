/*

let flotingVoteVersion0 = {

    // coud be imagine test net (it) or main net (im)
    net: "im",

    bVer: "0.0.0",
    bType: iConsts.BLOCK_TYPES.FVote,
    bCat: "",
    descriptions: null,
    // coud be imagine test net (it) or main net (im)
    blockLength: "0000000", // seialized block size by byte (char). this number is also a part of block root hash

    // root hash of all doc hashes
    // it is maked based on merkle tree root of transactions, segwits, wikis, SNSs, SSCs, DVCs, ...
    blockHash: "",
    confidence: 0.0, // dentos to signer/backer's shares by percent on signing time

    ancestors: "", // floating signatures MUST be linked to only one ancestor to whom which are signed

    voteData: {},    // the data which is voting for


    // the structure in which contain signature of shareholders(which are backers too) and
    // by their sign, they confirm the value of shares & DAG screen-shoot on that time.
    // later this confirmation will be used in validating the existance of a "sus-block" at the time of that cycle of coinbase
    //dExtInfo.signature: {}, // there must be only one signature of block backer/signer. it contains also backerAddress(the backer/signer address in which has DNA shares)

    // a list of ancestors blocks, these ancestor's hash also recorded in block root hash
    // if a block linked to an ancestors block, it must not liks to the father of that block
    // a <--- b <---- c
    // if the block linked to b, it must not link to a
    // the new block must be linked to as many possible as previous blocks (leave blocks)
    // maybe some discount depends on how many block you linked as an ancester!
    // when a new block linke t another block, it MUST not linked to that's block ancester


    // a list of coming feature to signaling whether the node supports or not
    signals: [], // e.g. "mimblewimble,taproot"

    // creation time timestamp, it is a part of block root-hash
    // it also used to calculate spendabality of an output. each output is not spendable befor passing 12 hours of creation time.
    // creation date must be greater than all it's ancesters
    creationDate: "", // 'yyyy-mm-dd hh:mm:ss'

};

// TODO: until missing vitality in imagine network and nodes, in case of facing the collision/dublespend/clone trxs...
// each node publishes a fVote Block and relay the other's fVotes(by flooding to entire neighbors).
// when network been more active, propagation of these blocks will overwhelm network,
// so we will put the block information inside a normal block as a fVote document
// in this case if block has other document to publish, will publish all in one block,
// other wise shuld wait a while and publish fVote block efficiently.

class fVoteHandler {


    // TODO: implent fVDoc in order to create a single document for floating votes and send the vote through a block.
    // in suche a way there is not need to spamming entire network for a single vote.
    // if in time of a half cycle, the machine didn't create a normal block and still has to present her vote(maybe because machine has a valuable shares)
    // then machine can send a legacy single floating vote block
    static createFVoteBlock(args) {
        let cDate = args.cDate;
        clog.app.info(`create FVote Block args: ${utils.stringify(args)}`);
        let msg;
        let uplink = args.uplink;   // the block which we are voting for

        let { percentage } = DNAHandler.getMachineShares();
        let minShare = cnfHandler.getMinShareToAllowedIssueFVote({ cDate });
        if (percentage < minShare) {
            msg = `machine hasn't sufficient shares (${percentage} < ${minShare}) to issue a Floting vote for any collision on block (${utils.hash6c(uplink)})`;
            clog.app.info(msg);
            // TODO: push fVote info to machine buffer in order to broadcost to network in first generated block.
            return { err: true, msg };
        }

        // create floating vote block
        let fVoteBlock = _.clone(flotingVoteVersion0);
        fVoteBlock.bCat = args.bCat;
        fVoteBlock.ancestors = [uplink]; // a sus-block-vote MUST HAVE ONLY ONE ancestors whom is going to be voted
        fVoteBlock.creationDate = utils.getNow();
        fVoteBlock.signals = iutils.getMachineSignals();
        fVoteBlock.confidence = percentage;
        fVoteBlock.voteData = args.voteData;

        let signMsg = this.getSignMsgBFVote(fVoteBlock);
        let { backer, uSet, signatures } = machine.signByMachineKey({ signMsg });
        fVoteBlock.bExtInfo = { uSet, signatures };
        fVoteBlock.backer = backer;
        fVoteBlock.bExtHash = iutils.doHashObject(fVoteBlock.bExtInfo);
        fVoteBlock.blockLength = iutils.offsettingLength(utils.stringify(fVoteBlock).length);
        fVoteBlock.blockHash = fVoteHandler.clacHashBFloatingVote(fVoteBlock);
        return { err: false, block: fVoteBlock };
    }

    static clacHashBFloatingVote(block) {
        // in order to have almost same hash! we sort the attribiutes alphabeticaly
        let hashableBlock = {
            backer: block.backer,
            bExtHash: block.bExtHash,
            blockLength: block.blockLength,
        };
        return iutils.doHashObject(hashableBlock);
    }

    static getSignMsgBFVote(block) {
        let signMsg = crypto.convertToSignMsg({
            ancestors: block.ancestors,
            bType: block.bType,
            bVer: block.bVer,
            creationDate: block.creationDate,
            confidence: block.confidence,
            net: block.net,
            voteData: block.voteData,
        });
        return signMsg;
    }



}


*/