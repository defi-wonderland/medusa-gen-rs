// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import { PropertiesA } from './PropertiesA.t.sol';
import { PropertiesB } from './PropertiesB.t.sol';

contract PropertiesParent is PropertiesA, PropertiesB {

}