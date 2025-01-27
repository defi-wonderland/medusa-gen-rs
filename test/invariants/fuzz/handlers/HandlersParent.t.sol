// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import { HandlersA } from './HandlersA.t.sol';
import { HandlersB } from './HandlersB.t.sol';

contract HandlersParent is HandlersA, HandlersB {

}