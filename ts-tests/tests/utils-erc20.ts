import Web3 from "web3";
import Contract from "web3-eth";
import { AbiItem } from "web3-utils";
import { createAndFinalizeBlock, describeWithFrontier, customRequest } from "./util";

import TestERC20 from "../build/contracts/TestERC20.json";

const TEST_CONTRACT_BYTECODE = TestERC20.bytecode;
const TEST_CONTRACT_ABI = TestERC20.abi as AbiItem[];

export async function createErc20Context(web3: Web3, from, from_key) {
	const tx = await web3.eth.accounts.signTransaction(
		{
			from: from,
			data: TEST_CONTRACT_BYTECODE,
			value: "0x00",
			gasPrice: "0x3B9ACA00",
			gas: "0x1000000",
		},
		from_key
	);
	await customRequest(web3, "eth_sendRawTransaction", [tx.rawTransaction]);
	await createAndFinalizeBlock(web3);

	// set the contract address
	let receipt0 = await web3.eth.getTransactionReceipt(tx.transactionHash);

	return receipt0.contractAddress;
}

export async function erc20Contract(web3: Web3, contract_address, from) {
	const contract = new web3.eth.Contract(TEST_CONTRACT_ABI, contract_address, {
		from: from,
		gasPrice: "0x3B9ACA00",
		gas: 100000,
	});

	return contract;
}

export async function erc20Transfer(web3: Web3, contract_address, key, from, to, token_value, nonce?: number) {
	const contract = await erc20Contract(web3, contract_address, from);

	const tx = await web3.eth.accounts.signTransaction(
		{
			from: from,
			to: contract_address,
			data: contract.methods.transfer(to, token_value).encodeABI(),
			value: "0x00",
			gasPrice: "0x3B9ACA00",
			gas: "0x500000",
			nonce: nonce,
		},
		key
	);

	await customRequest(web3, "eth_sendRawTransaction", [tx.rawTransaction]);

	return tx;
}

export async function erc20BalanceOf(web3: Web3, contract_address, account) {
	const contract = new web3.eth.Contract(TEST_CONTRACT_ABI, contract_address, {});

	return await contract.methods.balanceOf(account).call();
}
