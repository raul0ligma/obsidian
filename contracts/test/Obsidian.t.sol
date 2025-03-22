// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "src/ObsidianRouter.sol";

contract MockERC20 {
    string public name;
    string public symbol;
    uint8 private _decimals;
    uint256 private _totalSupply;

    mapping(address => uint256) private _balances;
    mapping(address => mapping(address => uint256)) private _allowances;

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(
        address indexed owner,
        address indexed spender,
        uint256 value
    );

    constructor(string memory name_, string memory symbol_, uint8 decimals_) {
        name = name_;
        symbol = symbol_;
        _decimals = decimals_;
    }

    function decimals() public view virtual returns (uint8) {
        return _decimals;
    }

    function balanceOf(address account) public view virtual returns (uint256) {
        return _balances[account];
    }

    function transfer(
        address to,
        uint256 amount
    ) public virtual returns (bool) {
        address owner = msg.sender;
        _transfer(owner, to, amount);
        return true;
    }

    function allowance(
        address owner,
        address spender
    ) public view virtual returns (uint256) {
        return _allowances[owner][spender];
    }

    function approve(
        address spender,
        uint256 amount
    ) public virtual returns (bool) {
        address owner = msg.sender;
        _approve(owner, spender, amount);
        return true;
    }

    function transferFrom(
        address from,
        address to,
        uint256 amount
    ) public virtual returns (bool) {
        address spender = msg.sender;
        _spendAllowance(from, spender, amount);
        _transfer(from, to, amount);
        return true;
    }

    function mint(address to, uint256 amount) public {
        _mint(to, amount);
    }

    function _transfer(
        address from,
        address to,
        uint256 amount
    ) internal virtual {
        require(from != address(0), "ERC20: transfer from the zero address");
        require(to != address(0), "ERC20: transfer to the zero address");

        uint256 fromBalance = _balances[from];
        require(
            fromBalance >= amount,
            "ERC20: transfer amount exceeds balance"
        );
        unchecked {
            _balances[from] = fromBalance - amount;
        }
        _balances[to] += amount;

        emit Transfer(from, to, amount);
    }

    function _mint(address account, uint256 amount) internal virtual {
        require(account != address(0), "ERC20: mint to the zero address");

        _totalSupply += amount;
        _balances[account] += amount;
        emit Transfer(address(0), account, amount);
    }

    function _approve(
        address owner,
        address spender,
        uint256 amount
    ) internal virtual {
        require(owner != address(0), "ERC20: approve from the zero address");
        require(spender != address(0), "ERC20: approve to the zero address");

        _allowances[owner][spender] = amount;
        emit Approval(owner, spender, amount);
    }

    function _spendAllowance(
        address owner,
        address spender,
        uint256 amount
    ) internal virtual {
        uint256 currentAllowance = allowance(owner, spender);
        if (currentAllowance != type(uint256).max) {
            require(
                currentAllowance >= amount,
                "ERC20: insufficient allowance"
            );
            unchecked {
                _approve(owner, spender, currentAllowance - amount);
            }
        }
    }
}

contract MockSP1Verifier is ISP1Verifier {
    mapping(bytes32 => mapping(bytes => mapping(bytes => bool)))
        private _verifyResults;

    function setVerifyProofResult(
        bytes32 vkey,
        bytes memory publicValues,
        bytes memory proofBytes,
        bool result
    ) external {
        _verifyResults[vkey][publicValues][proofBytes] = result;
    }

    function verifyProof(
        bytes32 vkey,
        bytes calldata publicValues,
        bytes calldata proofBytes
    ) external view override {
        require(
            _verifyResults[vkey][publicValues][proofBytes],
            "MockSP1Verifier: Verification failed"
        );
    }
}

contract ObsidianRouterTest is Test {
    address public constant SELLER = 0xd4f23AfEAcfc05399E58e122B9a23cD04FA02C3B;
    address public constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address public constant DAI = 0x6B175474E89094C44Da98b954EedeAC495271d0F;

    ObsidianRouter public router;
    MockSP1Verifier public verifier;
    address public buyer = address(0xBEEF);
    bytes32 public constant PROGRAM_VKEY = bytes32(uint256(0x123));

    function setUp() public {
        MockERC20 usdc = new MockERC20("USDC", "USDC", 6);
        MockERC20 dai = new MockERC20("DAI", "DAI", 18);

        vm.etch(USDC, address(usdc).code);
        vm.etch(DAI, address(dai).code);

        verifier = new MockSP1Verifier();
        router = new ObsidianRouter(address(verifier), PROGRAM_VKEY);

        MockERC20(DAI).mint(buyer, type(uint256).max);
        MockERC20(USDC).mint(SELLER, type(uint256).max);

        vm.prank(buyer);
        MockERC20(DAI).approve(address(router), type(uint256).max);

        vm.prank(SELLER);
        MockERC20(USDC).approve(address(router), type(uint256).max);
    }

    // function testSwap() public {
    //     bytes
    //         memory publicValues = hex"000000000000000000000000d4f23afeacfc05399e58e122b9a23cd04fa02c3b6e34099d8047d0e104db4c042d22c86e4365b45ceb1455f7ed05ed0654978da80000000000000000000000000000000000000000000000000000000001512d91000000000000000000000000000000000000000000000000456fe2c89d32bbde00000000000000000000000000000000000000000000000000000000004c4b400000000000000000000000006b175474e89094c44da98b954eedeac495271d0f000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";

    //     bytes
    //         memory proofBytes = hex"11b6a09d13bd9df98d0c96acdba53f7918caae67f2349200524346e30d61172f2bb6d0920d7891f2fc384fe93e5119fbd21960838aedafd182a9773b759fbee68db77ee50e16ec36d50a60c6cb3910ca3f8f8a79a76a41b88fceed1b1d3f5e8863741e412253542deaf43aefb7c8936666477c3d4cbc7668bad24a68e80ae28ad9b5f24617cee90007a243fc5a4e7bba773fb95c762799d03183bbe0d3e78c5c858eaf68139d7fba6134f634023e821a87b5615f2ea634cb43d6fde1a6dc61cfa2aea82624c823e70c2f12eb699088877ac5e8ec599c5d7454c151d36ee9a11a5acbb01616626cd9f0a52d3ca94ee90b49ce05416035b52d91dd146484c2aca350acfb66";

    //     verifier.setVerifyProofResult(
    //         PROGRAM_VKEY,
    //         publicValues,
    //         proofBytes,
    //         true
    //     );

    //     vm.prank(buyer);
    //     router.solve(publicValues, proofBytes);
    // }

    function testMainnetFork() public {
        uint256 mainnetFork = vm.createFork(vm.envString("MAINNET_RPC_URL"));
        vm.selectFork(mainnetFork);

        bytes
            memory publicValues = hex"000000000000000000000000d4f23afeacfc05399e58e122b9a23cd04fa02c3b8737e5f9d55e8d3bcba36b586dcef5ba6a032f108d6f99e04f446982d05a909e0000000000000000000000000000000000000000000000000000000001513c070000000000000000000000000000000000000000000000004553022b078fae8c00000000000000000000000000000000000000000000000000000000004c4b400000000000000000000000006b175474e89094c44da98b954eedeac495271d0f000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        bytes
            memory proofBytes = hex"11b6a09d1139b18757b0c5cb67153913a3f14601f942c1cd48cb144903b46820231cc0850eb3ae02692d7eb4e04c1047309a9955aa65e833ff218393094bf201a9d6629d18b3083e561207ec1d9507ae828a1816bd598deed135706759aee2875eec525415d47705ad335ffc1bb7ee607460aa0f39095c08686b1b48ac85050d17bc9d1b14acc7c281f090587f691ea3e7a6adc81414938000870af134cb06450f1c19550283dc25ba7c07e8e5287b0a9961630482a16c9f040006bb09c921b83adcf148227b65d25bf6a72f81db949fc22f4bd1b9959c3647636e63fef0c253e06847bf1dc71a1d1efd31bcf9559efaabad30b0ac97850e7dfa135e3447647b8d5942b8";

        ObsidianOrder memory order = abi.decode(publicValues, (ObsidianOrder));

        bytes32 vkey = hex"0087e1ea649d141c1b62fc8155f7df892a3ef29b9744fe46ded393fab786638b";

        // Option 1: Use real verifier if available on mainnet
        // address realVerifierAddress = 0x...; // Replace with actual verifier address
        // bytes32 realProgramVKey = bytes32(0x...); // Replace with actual program key
        // router = new ObsidianRouter(realVerifierAddress, realProgramVKey);

        // Option 2: Continue with mock verifier for testing (what we'll use)
        //verifier = new MockSP1Verifier();
        router = new ObsidianRouter(
            address(0x397A5f7f3dBd538f23DE225B51f532c34448dA9B),
            vkey
        );
        //verifier.setVerifyProofResult(vkey, PUBLIC_VALUES, PROOF_BYTES, true);

        // Fund our test addresses with tokens
        deal(DAI, buyer, 1000000e18);
        deal(USDC, order.seller, 500000000e6);

        // Set approvals
        vm.startPrank(buyer);
        IERC20(DAI).approve(address(router), type(uint256).max);
        vm.stopPrank();

        vm.startPrank(order.seller);
        IERC20(USDC).approve(address(router), type(uint256).max);
        vm.stopPrank();

        // Log initial balances
        console.log("========== Initial Balances ==========");
        console.log("Buyer DAI:", IERC20(DAI).balanceOf(buyer));
        console.log("Buyer USDC:", IERC20(USDC).balanceOf(buyer));
        console.log("Seller DAI:", IERC20(DAI).balanceOf(order.seller));
        console.log("Seller USDC:", IERC20(USDC).balanceOf(order.seller));

        // Execute swap
        vm.prank(buyer);
        router.solve(publicValues, proofBytes);

        // Log final balances
        console.log("========== Final Balances ==========");
        console.log("Buyer DAI:", IERC20(DAI).balanceOf(buyer));
        console.log("Buyer USDC:", IERC20(USDC).balanceOf(buyer));
        console.log("Seller DAI:", IERC20(DAI).balanceOf(order.seller));
        console.log("Seller USDC:", IERC20(USDC).balanceOf(order.seller));

        // Verify expected token movements
        assertEq(
            IERC20(DAI).balanceOf(order.seller),
            order.bought_amount,
            "Seller should receive bought_amount of DAI"
        );
        assertEq(
            IERC20(USDC).balanceOf(buyer),
            order.sold_amount,
            "Buyer should receive sold_amount of USDC"
        );

        console.log(" Mainnet fork swap successful!");
    }
}
