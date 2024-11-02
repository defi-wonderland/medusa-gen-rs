// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import { HandlerA } from './HandlerA.t.sol';
import { HandlerB } from './HandlerB.t.sol';

contract HandlerParent is HandlerA, HandlerB {

}