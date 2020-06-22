pragma solidity ^0.5.4;

interface Masternode {
    function register(bytes32, bytes32) external payable;
    function() payable external;
    function getInfo(bytes8) external view returns (
        bytes32 id1,
        bytes32 id2,
        bytes8 preId,
        bytes8 nextId,
        uint blockNumber,
        address account,
        uint blockOnlineAcc,
        uint blockLastPing
    );
    function has(bytes8) external view returns (bool);
    function voteForGovernanceAddress(address) external;
}

interface ProposalETZ {
    function vote(uint, uint) external;
}

contract FreeETZ {
    uint constant public MasternodeCost = 20000 * 10 ** 18;
    Masternode constant mn = Masternode(0x000000000000000000000000000000000000000A);
    
    struct node {
        bytes32 x;
        bytes32 y;
        address coinbase;
        MasternodeDelegate md;
        bool isValid;
    }
    mapping (bytes8 => node) public nodes;
    mapping (address => bytes8[]) public idsOf;
    bytes8[] public ids;
    uint public masternodeCount;
    
    address payable owner;
    address audit;
    
    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }

    modifier onlyAudit() {
        require(msg.sender == owner || msg.sender == audit);
        _;
    }
    
    constructor() public {
        owner = msg.sender;
    }   
    
    function() payable external {
        if(msg.value == MasternodeCost){
            bytes8 id = ids[masternodeCount];
            bytes32 x = nodes[id].x;
            bytes32 y = nodes[id].y;
            require(x != bytes32(0) && y != bytes32(0));
            MasternodeDelegate md = (new MasternodeDelegate).value(MasternodeCost)(x, y, msg.sender);
            nodes[id].coinbase = msg.sender;
            nodes[id].md = md;
            nodes[id].isValid = true;
            idsOf[msg.sender].push(id);
            masternodeCount++;
        }else if(msg.value == 0){
            for(uint i = 0; i < idsOf[msg.sender].length; i++){
                bytes8 id = idsOf[msg.sender][i];
                if(nodes[id].isValid){
                    nodes[id].md.quit();
                    nodes[id].isValid = false;
                    break;
                }
            }
        }else{
            revert();
        }
    }
    
    function withdraw(bytes8 id) public {
        nodes[id].md.withdraw();
    }
    
    function getCoinbase(bytes8 id) public view returns (address) {
       return nodes[id].coinbase;
    }    
    
    function getIds(address addr, uint startPos) public view 
    returns (uint length, bytes8[5] memory data) {
        bytes8[] memory myIds = idsOf[addr];
        length = uint(myIds.length);
        for(uint i = 0; i < 5 && (i+startPos) < length; i++) {
            data[i] = myIds[i+startPos];
        }
    }
    
    function voteGA(bytes8 id, address addr) onlyAudit public{
        nodes[id].md.voteGA(addr);
    }

    function vote(bytes8 id, uint index, uint  voteType) onlyAudit public {
        nodes[id].md.vote(index, voteType);
    }
    
    function setOwner(address payable addr) onlyOwner public {
        require(addr != address(0) && addr != owner);
        owner = addr;
    }

    function setRes(bytes32[] memory data) public onlyAudit {
        for (uint i=0; i<data.length; i=i+2){
            bytes8 id = bytes8(data[i]);
            if(id != bytes8(0) && nodes[id].x == bytes32(0)) {
                ids.push(id);
                nodes[id].x = data[i];
                nodes[id].y = data[i+1];
            }
        }
    }

    function resetRes(uint index, bytes32 x, bytes32 y) public onlyAudit {
        bytes8 id = ids[index];
        if(id != bytes8(0) && nodes[id].coinbase == address(0)) {
            ids[index] = bytes8(x);
            nodes[id].x = x;
            nodes[id].y = y;
        }
    }

    function restResCount() public view returns(uint len) {
        return ids.length - masternodeCount;
    }

}

// Masternode Delegate
contract MasternodeDelegate {
    Masternode mn = Masternode(0x000000000000000000000000000000000000000A);

    address public owner;
    address payable public _coinbase;

    modifier auth() {
        require(msg.sender == owner || msg.sender == _coinbase);
        _;
    }

    constructor(bytes32 id1, bytes32 id2, address payable coinbase) public payable {
        owner = msg.sender;
        _coinbase = coinbase;
        mn.register.value(msg.value)(id1, id2);
    }

    function() payable external {}

    function quit() payable auth public returns(bool) {
        (bool rt, ) = address(mn).call.value(0)("");
        if(rt) _coinbase.transfer(address(this).balance);
        return rt;
    }
    
    function withdraw() payable public {
        _coinbase.transfer(address(this).balance);
    }
    
    function voteGA(address addr) auth public{
        Masternode(mn).voteForGovernanceAddress(addr);
    }

    function vote(uint index, uint  voteType) auth public {
        ProposalETZ(0x4761977f757E3031350612D55bb891c8144a414B).vote(index, voteType);
    }
}