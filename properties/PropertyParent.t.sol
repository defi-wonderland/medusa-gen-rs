// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import { PropertyA } from './PropertyA.t.sol';
import { PropertyB } from './PropertyB.t.sol';

contract PropertyParent is PropertyA, PropertyB {

}