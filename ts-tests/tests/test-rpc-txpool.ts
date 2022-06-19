import { expect } from "chai";
import { step } from "mocha-steps";
import {
    createAndFinalizeBlock,
    describeWithFrontier,
} from "./util";
import {
    createErc20Context,
    erc20BalanceOf,
    erc20Transfer,
} from "./utils-erc20";

import TestERC20 from "../build/contracts/TestERC20.json";

describeWithFrontier("ETH RPC(Txpool For ERC20)", (context) => {
    const GENESIS_ACCOUNT = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
    const GENESIS_ACCOUNT_PRIVATE_KEY =
        "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
    const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
    let CONTRACT_ADDRESS = "0x00"; // Those test are ordered. In general this should be avoided, but due to the time it takes	// to spin up a frontier node, it saves a lot of time.

    before("create the contract", async function () {
        this.timeout(15000);
        CONTRACT_ADDRESS = await createErc20Context(
            context.web3,
            GENESIS_ACCOUNT,
            GENESIS_ACCOUNT_PRIVATE_KEY
        );
    });

    step("Get txpool content rpc", async function () {
        this.timeout(15000);

        for (let i of [1, 2, 3]) {
            await erc20Transfer(
                context.web3,
                CONTRACT_ADDRESS,
                GENESIS_ACCOUNT_PRIVATE_KEY,
                GENESIS_ACCOUNT,
                TEST_ACCOUNT,
                1000,
                i
            );
        }

        let pending = await context.web3["txpool"].content();

        // console.log(JSON.stringify(pending, null, 2));

        expect(pending.pending).to.be.eql({
            "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac": {
                "0x1": {
                    blockHash:
                        "0x0000000000000000000000000000000000000000000000000000000000000000",
                    blockNumber: null,
                    hash: "0x8cb6c154bddeda606be33282cbebbf7745c3a61a306f89bc31dd40ba6b24be7b",
                    transactionIndex: null,
                    from: "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac",
                    to: "0xc01ee7f10ea4af4673cfff62710e1d7792aba8f3",
                    nonce: "0x1",
                    value: "0x0",
                    gas: "0x500000",
                    gasPrice: "0x3b9aca00",
                    input: "0xa9059cbb000000000000000000000000111111111111111111111111111111111111111100000000000000000000000000000000000000000000000000000000000003e8",
                },
                "0x3": {
                    blockHash:
                        "0x0000000000000000000000000000000000000000000000000000000000000000",
                    blockNumber: null,
                    hash: "0x60609aa3ed063d850d987e6a9b7190d8b3d5c8457b40108133832b0546b3ff6c",
                    transactionIndex: null,
                    from: "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac",
                    to: "0xc01ee7f10ea4af4673cfff62710e1d7792aba8f3",
                    nonce: "0x3",
                    value: "0x0",
                    gas: "0x500000",
                    gasPrice: "0x3b9aca00",
                    input: "0xa9059cbb000000000000000000000000111111111111111111111111111111111111111100000000000000000000000000000000000000000000000000000000000003e8",
                },
                "0x2": {
                    blockHash:
                        "0x0000000000000000000000000000000000000000000000000000000000000000",
                    blockNumber: null,
                    hash: "0xab7a8b1f20c6ffbf3ad5cd4e783e5e413ed171f9bb3fa0a2a26a5cbbf86ecbea",
                    transactionIndex: null,
                    from: "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac",
                    to: "0xc01ee7f10ea4af4673cfff62710e1d7792aba8f3",
                    nonce: "0x2",
                    value: "0x0",
                    gas: "0x500000",
                    gasPrice: "0x3b9aca00",
                    input: "0xa9059cbb000000000000000000000000111111111111111111111111111111111111111100000000000000000000000000000000000000000000000000000000000003e8",
                },
            },
        });

        await createAndFinalizeBlock(context.web3);

        expect(
            await erc20BalanceOf(context.web3, CONTRACT_ADDRESS, TEST_ACCOUNT)
        ).to.be.eq("3000");

        let null_res = await context.web3["txpool"].content();

        // console.log(JSON.stringify(null_res, null, 2));

        expect(null_res.pending).to.deep.equal({});
        expect(null_res.queued).to.deep.equal({});

        for (let i of [4, 5, 7]) {
            await erc20Transfer(
                context.web3,
                CONTRACT_ADDRESS,
                GENESIS_ACCOUNT_PRIVATE_KEY,
                GENESIS_ACCOUNT,
                TEST_ACCOUNT,
                1000,
                i
            );
        }

        let queued = await context.web3["txpool"].content();

        // console.log(JSON.stringify(queued, null, 2));

        expect(queued).to.be.eql({
            pending: {
                "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac": {
                    "0x5": {
                        blockHash:
                            "0x0000000000000000000000000000000000000000000000000000000000000000",
                        blockNumber: null,
                        hash: "0x6a7ae7ccb9fd01c6808b9c11911ed8680744a483f6053d45399b1cecacd3c5df",
                        transactionIndex: null,
                        from: "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac",
                        to: "0xc01ee7f10ea4af4673cfff62710e1d7792aba8f3",
                        nonce: "0x5",
                        value: "0x0",
                        gas: "0x500000",
                        gasPrice: "0x3b9aca00",
                        input: "0xa9059cbb000000000000000000000000111111111111111111111111111111111111111100000000000000000000000000000000000000000000000000000000000003e8",
                    },
                    "0x4": {
                        blockHash:
                            "0x0000000000000000000000000000000000000000000000000000000000000000",
                        blockNumber: null,
                        hash: "0x2acd398569e50a6a3e6ed803c72e2cd4f06a49d717beaaf94e0f1a582a101d7f",
                        transactionIndex: null,
                        from: "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac",
                        to: "0xc01ee7f10ea4af4673cfff62710e1d7792aba8f3",
                        nonce: "0x4",
                        value: "0x0",
                        gas: "0x500000",
                        gasPrice: "0x3b9aca00",
                        input: "0xa9059cbb000000000000000000000000111111111111111111111111111111111111111100000000000000000000000000000000000000000000000000000000000003e8",
                    },
                },
            },
            queued: {
                "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac": {
                    "0x7": {
                        blockHash:
                            "0x0000000000000000000000000000000000000000000000000000000000000000",
                        blockNumber: null,
                        hash: "0x113fe4d35150ac3f0651f00ff29151c90500f8268544f6ddb18b0884a0d2604f",
                        transactionIndex: null,
                        from: "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac",
                        to: "0xc01ee7f10ea4af4673cfff62710e1d7792aba8f3",
                        nonce: "0x7",
                        value: "0x0",
                        gas: "0x500000",
                        gasPrice: "0x3b9aca00",
                        input: "0xa9059cbb000000000000000000000000111111111111111111111111111111111111111100000000000000000000000000000000000000000000000000000000000003e8",
                    },
                },
            },
        });

        await createAndFinalizeBlock(context.web3);

        let queued_after = await context.web3["txpool"].content();

        // console.log(JSON.stringify(queued_after, null, 2));

        expect(queued_after).to.be.eql({
            pending: {},
            queued: {
                "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac": {
                    "0x7": {
                        blockHash:
                            "0x0000000000000000000000000000000000000000000000000000000000000000",
                        blockNumber: null,
                        hash: "0x113fe4d35150ac3f0651f00ff29151c90500f8268544f6ddb18b0884a0d2604f",
                        transactionIndex: null,
                        from: "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac",
                        to: "0xc01ee7f10ea4af4673cfff62710e1d7792aba8f3",
                        nonce: "0x7",
                        value: "0x0",
                        gas: "0x500000",
                        gasPrice: "0x3b9aca00",
                        input: "0xa9059cbb000000000000000000000000111111111111111111111111111111111111111100000000000000000000000000000000000000000000000000000000000003e8",
                    },
                },
            },
        });
    });
});
