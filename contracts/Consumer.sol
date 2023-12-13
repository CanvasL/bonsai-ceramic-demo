// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import {IBonsaiRelay} from "bonsai/IBonsaiRelay.sol";
import {BonsaiCallbackReceiver} from "bonsai/BonsaiCallbackReceiver.sol";

contract Consumer is BonsaiCallbackReceiver {
    event QuerySent(bytes32 imageId, bytes requestData, address callbackAddr, bytes4 callbackFunc, uint64 gasLimit);
    event QueryFulfilled(bytes32 imageId);

    error ImageUnmatched();

    uint64 private constant BONSAI_CALLBACK_GAS_LIMIT = 100000;

    bytes32 internal _imageId;

    /**
     * @dev dappId => fileId => latestFileContent bytes
     */
    mapping(bytes16 => mapping(string => bytes)) _latestFileContent;

    constructor(address bonsaiRelay, bytes32 imageId) BonsaiCallbackReceiver(IBonsaiRelay(bonsaiRelay)) {
        _imageId = imageId;
    }

    function queryLatestFileContent(bytes16 dappId, string calldata fileId) external {
        bytes memory requestData = abi.encode(dappId, fileId);
        emit QuerySent(_imageId, requestData, address(this), this.fulfillLatestFileContent.selector, BONSAI_CALLBACK_GAS_LIMIT);
    }

    function fulfillLatestFileContent(bytes calldata journal, bytes32 imageId) external onlyBonsaiCallback(_imageId) {
        if(imageId != _imageId) {
            revert ImageUnmatched();
        }
        (bytes16 dappId, string memory fileId, bytes memory result) = abi.decode(journal, (bytes16, string, bytes));
        _latestFileContent[dappId][fileId] = result;
        emit QueryFulfilled(imageId);
    }
}