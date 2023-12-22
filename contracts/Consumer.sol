// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import {IBonsaiRelay} from "bonsai/IBonsaiRelay.sol";
import {BonsaiCallbackReceiver} from "bonsai/BonsaiCallbackReceiver.sol";

contract Consumer is BonsaiCallbackReceiver {
    event QuerySent(bytes32 queryId, bytes32 imageId, bytes queryData, address callbackAddr, bytes4 callbackFunc, uint64 gasLimit);
    event QueryFulfilled(bytes32 queryId);

    error ImageUnmatched();

    uint64 public constant BONSAI_CALLBACK_GAS_LIMIT = 100000;

    bytes32 public IMAGE_ID;
    mapping(address => mapping(string => mapping(string => bool))) public commitValidity;
    mapping(bytes32 => bool) _queryFulfilled;

    constructor(address bonsaiRelay, bytes32 imageId) BonsaiCallbackReceiver(IBonsaiRelay(bonsaiRelay)) {
        IMAGE_ID = imageId;
    }

    function isQueryFulfilled(bytes32 queryId) external view returns (bool) {
        return _queryFulfilled[queryId];
    }

    function queryStreamFileValidity(address owner, string calldata fileId, string calldata commitId) external {
        bytes memory queryData = abi.encode(owner, fileId, commitId);
        bytes32 queryId = keccak256(abi.encodePacked(queryData, address(this), this.fulfillQueryStreamFileValidity.selector, BONSAI_CALLBACK_GAS_LIMIT));
        emit QuerySent(queryId, IMAGE_ID, queryData, address(this), this.fulfillQueryStreamFileValidity.selector, BONSAI_CALLBACK_GAS_LIMIT);
    }

    function fulfillQueryStreamFileValidity(bytes32 queryId, address owner, string memory fileId, string memory commitId, bool result) external onlyBonsaiCallback(IMAGE_ID) {
        commitValidity[owner][fileId][commitId] = result;
        _queryFulfilled[queryId];

        emit QueryFulfilled(queryId);
    }
}