// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.7.0 <0.9.0;

/*
    Sonobe's Nova + CycleFold decider verifier.
    Joint effort by 0xPARC & PSE.

    More details at https://github.com/privacy-scaling-explorations/sonobe
    Usage and design documentation at https://privacy-scaling-explorations.github.io/sonobe-docs/

    Uses the https://github.com/iden3/snarkjs/blob/master/templates/verifier_groth16.sol.ejs
    Groth16 verifier implementation and a KZG10 Solidity template adapted from
    https://github.com/weijiekoh/libkzg.
    Additionally we implement the NovaDecider contract, which combines the
    Groth16 and KZG10 verifiers to verify the zkSNARK proofs coming from
    Nova+CycleFold folding.
*/


/* =============================== */
/* KZG10 verifier methods */
/**
 * @author  Privacy and Scaling Explorations team - pse.dev
 * @dev     Contains utility functions for ops in BN254; in G_1 mostly.
 * @notice  Forked from https://github.com/weijiekoh/libkzg.
 * Among others, a few of the changes we did on this fork were:
 * - Templating the pragma version
 * - Removing type wrappers and use uints instead
 * - Performing changes on arg types
 * - Update some of the `require` statements 
 * - Use the bn254 scalar field instead of checking for overflow on the babyjub prime
 * - In batch checking, we compute auxiliary polynomials and their commitments at the same time.
 */
contract KZG10Verifier {

    // prime of field F_p over which y^2 = x^3 + 3 is defined
    uint256 public constant BN254_PRIME_FIELD =
        21888242871839275222246405745257275088696311157297823662689037894645226208583;
    uint256 public constant BN254_SCALAR_FIELD =
        21888242871839275222246405745257275088548364400416034343698204186575808495617;

    /**
     * @notice  Performs scalar multiplication in G_1.
     * @param   p  G_1 point to multiply
     * @param   s  Scalar to multiply by
     * @return  r  G_1 point p multiplied by scalar s
     */
    function mulScalar(uint256[2] memory p, uint256 s) internal view returns (uint256[2] memory r) {
        uint256[3] memory input;
        input[0] = p[0];
        input[1] = p[1];
        input[2] = s;
        bool success;
        assembly {
            success := staticcall(sub(gas(), 2000), 7, input, 0x60, r, 0x40)
            switch success
            case 0 { invalid() }
        }
        require(success, "bn254: scalar mul failed");
    }

    /**
     * @notice  Negates a point in G_1.
     * @param   p  G_1 point to negate
     * @return  uint256[2]  G_1 point -p
     */
    function negate(uint256[2] memory p) internal pure returns (uint256[2] memory) {
        if (p[0] == 0 && p[1] == 0) {
            return p;
        }
        return [p[0], BN254_PRIME_FIELD - (p[1] % BN254_PRIME_FIELD)];
    }

    /**
     * @notice  Adds two points in G_1.
     * @param   p1  G_1 point 1
     * @param   p2  G_1 point 2
     * @return  r  G_1 point p1 + p2
     */
    function add(uint256[2] memory p1, uint256[2] memory p2) internal view returns (uint256[2] memory r) {
        bool success;
        uint256[4] memory input = [p1[0], p1[1], p2[0], p2[1]];
        assembly {
            success := staticcall(sub(gas(), 2000), 6, input, 0x80, r, 0x40)
            switch success
            case 0 { invalid() }
        }

        require(success, "bn254: point add failed");
    }

    /**
     * @notice  Computes the pairing check e(p1, p2) * e(p3, p4) == 1
     * @dev     Note that G_2 points a*i + b are encoded as two elements of F_p, (a, b)
     * @param   a_1  G_1 point 1
     * @param   a_2  G_2 point 1
     * @param   b_1  G_1 point 2
     * @param   b_2  G_2 point 2
     * @return  result  true if pairing check is successful
     */
    function pairing(uint256[2] memory a_1, uint256[2][2] memory a_2, uint256[2] memory b_1, uint256[2][2] memory b_2)
        internal
        view
        returns (bool result)
    {
        uint256[12] memory input = [
            a_1[0],
            a_1[1],
            a_2[0][1], // imaginary part first
            a_2[0][0],
            a_2[1][1], // imaginary part first
            a_2[1][0],
            b_1[0],
            b_1[1],
            b_2[0][1], // imaginary part first
            b_2[0][0],
            b_2[1][1], // imaginary part first
            b_2[1][0]
        ];

        uint256[1] memory out;
        bool success;

        assembly {
            success := staticcall(sub(gas(), 2000), 8, input, 0x180, out, 0x20)
            switch success
            case 0 { invalid() }
        }

        require(success, "bn254: pairing failed");

        return out[0] == 1;
    }

    uint256[2] G_1 = [
            20915074708566661972053837919571830680167797160963132225358039887599542502886,
            19707649967277603392022762435321869330251630284499931656630849082237705195504
    ];
    uint256[2][2] G_2 = [
        [
            14549452121285784778175827852087742256459617731502923490845797379162966393277,
            7282286194068975522791707748301048172794134290481483384938618350604060353532
        ],
        [
            21302967558482307629688192440155668184777079483771886327506175319264482728621,
            20429357755146558546576383805130941621639393612741738912493621400969974124755
        ]
    ];
    uint256[2][2] VK = [
        [
            8484572917391808915555752450838435622786016310153352964699769588432688029048,
            7096883417332348739819020399899464132850886857258526730978230732108993960885
        ],
        [
            6599615132638690615807602261424626420343919463498591582078459774107193040597,
            16397349615193781367988457718548903978450136896022507165640968511279008921791
        ]
    ];

    

    /**
     * @notice  Verifies a single point evaluation proof. Function name follows `ark-poly`.
     * @dev     To avoid ops in G_2, we slightly tweak how the verification is done.
     * @param   c  G_1 point commitment to polynomial.
     * @param   pi G_1 point proof.
     * @param   x  Value to prove evaluation of polynomial at.
     * @param   y  Evaluation poly(x).
     * @return  result Indicates if KZG proof is correct.
     */
    function check(uint256[2] calldata c, uint256[2] calldata pi, uint256 x, uint256 y)
        public
        view
        returns (bool result)
    {
        //
        // we want to:
        //      1. avoid gas intensive ops in G2
        //      2. format the pairing check in line with what the evm opcode expects.
        //
        // we can do this by tweaking the KZG check to be:
        //
        //          e(pi, vk - x * g2) = e(c - y * g1, g2) [initial check]
        //          e(pi, vk - x * g2) * e(c - y * g1, g2)^{-1} = 1
        //          e(pi, vk - x * g2) * e(-c + y * g1, g2) = 1 [bilinearity of pairing for all subsequent steps]
        //          e(pi, vk) * e(pi, -x * g2) * e(-c + y * g1, g2) = 1
        //          e(pi, vk) * e(-x * pi, g2) * e(-c + y * g1, g2) = 1
        //          e(pi, vk) * e(x * -pi - c + y * g1, g2) = 1 [done]
        //                        |_   rhs_pairing  _|
        //
        uint256[2] memory rhs_pairing =
            add(mulScalar(negate(pi), x), add(negate(c), mulScalar(G_1, y)));
        return pairing(pi, VK, rhs_pairing, G_2);
    }

    function evalPolyAt(uint256[] memory _coefficients, uint256 _index) public pure returns (uint256) {
        uint256 m = BN254_SCALAR_FIELD;
        uint256 result = 0;
        uint256 powerOfX = 1;

        for (uint256 i = 0; i < _coefficients.length; i++) {
            uint256 coeff = _coefficients[i];
            assembly {
                result := addmod(result, mulmod(powerOfX, coeff, m), m)
                powerOfX := mulmod(powerOfX, _index, m)
            }
        }
        return result;
    }

    
}

/* =============================== */
/* Groth16 verifier methods */
/*
    Copyright 2021 0KIMS association.

    * `solidity-verifiers` added comment
        This file is a template built out of [snarkJS](https://github.com/iden3/snarkjs) groth16 verifier.
        See the original ejs template [here](https://github.com/iden3/snarkjs/blob/master/templates/verifier_groth16.sol.ejs)
    *

    snarkJS is a free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    snarkJS is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
    or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public
    License for more details.

    You should have received a copy of the GNU General Public License
    along with snarkJS. If not, see <https://www.gnu.org/licenses/>.
*/

contract Groth16Verifier {
    // Scalar field size
    uint256 constant r    = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    // Base field size
    uint256 constant q   = 21888242871839275222246405745257275088696311157297823662689037894645226208583;

    // Verification Key data
    uint256 constant alphax  = 17889018629755954757275313837720145749586934028657754358049349212127030039593;
    uint256 constant alphay  = 20070931592534849667606556786147111383846780634323683504885682728856376364435;
    uint256 constant betax1  = 7091025299418744030992141656734677697921523274520995247953525136983396774078;
    uint256 constant betax2  = 20935759373994769381829829644100626715268397554912023663497817830310884435686;
    uint256 constant betay1  = 10567140606090650866167527985528866020245688229363044012278100943828976433123;
    uint256 constant betay2  = 3053517701034710280099267127961865895356865607019557128130339704925037318313;
    uint256 constant gammax1 = 8550021517717965508612766902513317248705497511049344898989604320389012601127;
    uint256 constant gammax2 = 8305567211301548972568302043515458004701753289439375569782483218894492755968;
    uint256 constant gammay1 = 2462267493947166989657416084108241228733304142157572978231194411678629862390;
    uint256 constant gammay2 = 2770309423401672675800714312364422309221971193392466043439597390453230342466;
    uint256 constant deltax1 = 13306426753422036788571345681333557262652931582933510589534937917946017990382;
    uint256 constant deltax2 = 15028573731798291861088032418960628282032371202883597368194445243978280574426;
    uint256 constant deltay1 = 7399515210976534908741172029697557114638470050883512189134611616829995131173;
    uint256 constant deltay2 = 14634366204502154763399324467061375958095771867327175998345329529014537413718;

    
    uint256 constant IC0x = 3800135551691338385104904639390642305172782143534269868520042700405657187195;
    uint256 constant IC0y = 19745852486996471455230591055948013823549303790357743637962500595530952192542;
    
    uint256 constant IC1x = 111802758993187207412379570332214147834615154649051366578106616616253048935;
    uint256 constant IC1y = 8326834410391207868227151337544048190347751772356972388875055501032611278864;
    
    uint256 constant IC2x = 974098787644128465360206818554480462072219602156085640015616320094675969748;
    uint256 constant IC2y = 18856988152205251254050332033831575103636268128507487326203377972897876602821;
    
    uint256 constant IC3x = 3226209202295839754181244181698652397417387515339982295551809074048833245151;
    uint256 constant IC3y = 8416470537691420863410239678230808022331375322980421248467671703255255757617;
    
    uint256 constant IC4x = 4214914031781722184900231312146659025188406492215401064317579482891914767132;
    uint256 constant IC4y = 8479550863604097761066553951044200369738944144319364758726202646738514381236;
    
    uint256 constant IC5x = 11310485347201692296899951433270887608404912690020350534741279162702304090361;
    uint256 constant IC5y = 2492255112593444395308361905739396994121388965323569581394030453524542099890;
    
    uint256 constant IC6x = 17572751875488171509112944280578525109941950077395869877110928253718321302248;
    uint256 constant IC6y = 21287913691362100432332841234148992011690723581969686942476977483455316929958;
    
    uint256 constant IC7x = 16746385949948151920894629077585252306782356582106883413361100918954271396106;
    uint256 constant IC7y = 2017925509435576844309631072337060065093187032172633975264469956789627333191;
    
    uint256 constant IC8x = 4078218291013526573514914620399747380691099907385216489120999870806388772358;
    uint256 constant IC8y = 3060969978064972354668532175261799643458255570815132658156322188868476242613;
    
    uint256 constant IC9x = 11991335064253045137062184252904402099094117463739358013027651704609717134977;
    uint256 constant IC9y = 15836227156464879331407744697594032141049212096702126723748003294035828692286;
    
    uint256 constant IC10x = 13294124675969148591401260050254970050011282887133802794064842185034498693612;
    uint256 constant IC10y = 4028241508861662206970839179442548391144067460640741786247936239579498538860;
    
    uint256 constant IC11x = 10317298630127185023234679850602993533885157319146417401089845662800929926245;
    uint256 constant IC11y = 10246201244057812860697224439633504232896540680182428549027337508761408627149;
    
    uint256 constant IC12x = 13443466300246667369104293385934046297589081604656539690242596930714775372234;
    uint256 constant IC12y = 10155028610444141115870533601321612042965109529640706958403009415179008589837;
    
    uint256 constant IC13x = 3751225488086353018974583871388906666774135138582155710844776781208103104557;
    uint256 constant IC13y = 10503217503228995414984759293507762608858882877931741916901150557014084100056;
    
    uint256 constant IC14x = 17998694455889111597248362015843699599196267426092351478896361805299797692963;
    uint256 constant IC14y = 11957245552817575723584471417806044285152344977868442247585205765520173619885;
    
    uint256 constant IC15x = 5470144808704454938662560252300819266973173105253434376662398523392686667589;
    uint256 constant IC15y = 7440664844377116292455859945662252304696009295325788782860946981232063011803;
    
    uint256 constant IC16x = 1471526429723992832358275695413449961626360240608782176578688807388094047945;
    uint256 constant IC16y = 13841267776529637459077908967096113818500967805628572230829031314431745403254;
    
    uint256 constant IC17x = 531588079005253639364828928118865480707595874907874259817132779732718595084;
    uint256 constant IC17y = 15547640383944753382009177614320105740940641605179226562788615893024062239033;
    
    uint256 constant IC18x = 6731737758859809623674396966206251799274078956000555554322870640469575232996;
    uint256 constant IC18y = 2238448075643341956860721703243500185038041708075302548235195054468615078747;
    
    uint256 constant IC19x = 11167204084142542772577419424973502575188214763367077427502096437514277108021;
    uint256 constant IC19y = 4720632421476710005088099346271873843415118670389214603164105412875610516687;
    
    uint256 constant IC20x = 19583544558305615991117119977674720852439201832985061084752910509765289530368;
    uint256 constant IC20y = 19777809599064866364703831743270568937716577026796821202295156060799852975114;
    
    uint256 constant IC21x = 15365053327601305772433621164764329668800099709000844285460864237876963597815;
    uint256 constant IC21y = 20669402317832098640369324947432539048167922023153908658360848438200114870622;
    
    uint256 constant IC22x = 13696808577338080377768225962746309732371041725772773429840968772840540196721;
    uint256 constant IC22y = 7990466593160262986149897059533434336397837039879118261665922097841993436368;
    
    uint256 constant IC23x = 17517262809899226004685851417747417723279819648750066406464465647744497391647;
    uint256 constant IC23y = 9917399218371577356168104770182190916416128411495164942105781562915329530724;
    
    uint256 constant IC24x = 8472355822736407414403174156227132292684661065995860357145122877190753713412;
    uint256 constant IC24y = 524208698850051034172946956299796141841827722763054723006964569889573143390;
    
    uint256 constant IC25x = 5045075350938642328879759307060949820161904111053879826115624276393026590141;
    uint256 constant IC25y = 6302447802443812071999421070494300476729123770642845683995973997365461790804;
    
    uint256 constant IC26x = 2762614029503475612829366049868087041580118094002468242477293613182488240091;
    uint256 constant IC26y = 9310873701572678185869539661742047678256916738541170464825882691148908117046;
    
    uint256 constant IC27x = 2070571261610581409871271922630779730501109960659141016621315001840305486196;
    uint256 constant IC27y = 14820140577743343871185987551722957448867608790376870911570171079257287622355;
    
    uint256 constant IC28x = 447131243985619502066034932890821549843887492833175627365572995592001486163;
    uint256 constant IC28y = 15384584355466759580047019307141400627520144353790208648418525681096432547187;
    
    uint256 constant IC29x = 6820507477988595594615979419189153155471313905410809569554453104769710559169;
    uint256 constant IC29y = 6133065798513398899905942589725191166581285853166994125104332969154646069521;
    
    uint256 constant IC30x = 19416865014299325086508380708761024497041274003943304311922329722347737413148;
    uint256 constant IC30y = 16792626803419878854141398411441861969131849495383375858093453452892385081913;
    
    uint256 constant IC31x = 6132202339867885457342975939612805507779037445166064321344867588169383314780;
    uint256 constant IC31y = 15092587388007988940864766289364504724578369217850254628598679990036023589509;
    
    uint256 constant IC32x = 13130662665395395387536599930349293998400350980114492073268378637562352625022;
    uint256 constant IC32y = 1321161714262703489252710432191567566358061677711332879378664015698759873071;
    
    uint256 constant IC33x = 942995547950810519051916763156433847252855489067839800556325072596656733313;
    uint256 constant IC33y = 19591078383800448581107746187734927604610565362422624310540610577312726961173;
    
    uint256 constant IC34x = 10646740380723990127203076621712100726944386600585664890641010251953101911415;
    uint256 constant IC34y = 11813765588738729708736904538996409021965110567763529460554353754111969507911;
    
    uint256 constant IC35x = 19007545146089090355492903989301006565631193262148592264356363757713022041877;
    uint256 constant IC35y = 8467892866603267217072014457922552049824812286478477057882730627448597514640;
    
    uint256 constant IC36x = 16510662462503407761778258261372546949512730214059140975998335390613579675304;
    uint256 constant IC36y = 29061120915039885802661809418701308194123573732729154779999137804947539646;
    
    uint256 constant IC37x = 6960287479842363182008274991870560483397897656535864078363674969891071817698;
    uint256 constant IC37y = 6008044309941311279511672323297086139730746738744698678212245794241904740302;
    
    uint256 constant IC38x = 3418070897850056226430975307889923847985467836244607134662067503375789550380;
    uint256 constant IC38y = 12694731027304419978302927756716240711108687128982813785743898939715166480511;
    
    
    // Memory data
    uint16 constant pVk = 0;
    uint16 constant pPairing = 128;

    uint16 constant pLastMem = 896;

    function verifyProof(uint[2] calldata _pA, uint[2][2] calldata _pB, uint[2] calldata _pC, uint[38] calldata _pubSignals) public view returns (bool) {
        assembly {
            function checkField(v) {
                if iszero(lt(v, r)) {
                    mstore(0, 0)
                    return(0, 0x20)
                }
            }
            
            // G1 function to multiply a G1 value(x,y) to value in an address
            function g1_mulAccC(pR, x, y, s) {
                let success
                let mIn := mload(0x40)
                mstore(mIn, x)
                mstore(add(mIn, 32), y)
                mstore(add(mIn, 64), s)

                success := staticcall(sub(gas(), 2000), 7, mIn, 96, mIn, 64)

                if iszero(success) {
                    mstore(0, 0)
                    return(0, 0x20)
                }

                mstore(add(mIn, 64), mload(pR))
                mstore(add(mIn, 96), mload(add(pR, 32)))

                success := staticcall(sub(gas(), 2000), 6, mIn, 128, pR, 64)

                if iszero(success) {
                    mstore(0, 0)
                    return(0, 0x20)
                }
            }

            function checkPairing(pA, pB, pC, pubSignals, pMem) -> isOk {
                let _pPairing := add(pMem, pPairing)
                let _pVk := add(pMem, pVk)

                mstore(_pVk, IC0x)
                mstore(add(_pVk, 32), IC0y)

                // Compute the linear combination vk_x
                
                
                g1_mulAccC(_pVk, IC1x, IC1y, calldataload(add(pubSignals, 0)))
                g1_mulAccC(_pVk, IC2x, IC2y, calldataload(add(pubSignals, 32)))
                g1_mulAccC(_pVk, IC3x, IC3y, calldataload(add(pubSignals, 64)))
                g1_mulAccC(_pVk, IC4x, IC4y, calldataload(add(pubSignals, 96)))
                g1_mulAccC(_pVk, IC5x, IC5y, calldataload(add(pubSignals, 128)))
                g1_mulAccC(_pVk, IC6x, IC6y, calldataload(add(pubSignals, 160)))
                g1_mulAccC(_pVk, IC7x, IC7y, calldataload(add(pubSignals, 192)))
                g1_mulAccC(_pVk, IC8x, IC8y, calldataload(add(pubSignals, 224)))
                g1_mulAccC(_pVk, IC9x, IC9y, calldataload(add(pubSignals, 256)))
                g1_mulAccC(_pVk, IC10x, IC10y, calldataload(add(pubSignals, 288)))
                g1_mulAccC(_pVk, IC11x, IC11y, calldataload(add(pubSignals, 320)))
                g1_mulAccC(_pVk, IC12x, IC12y, calldataload(add(pubSignals, 352)))
                g1_mulAccC(_pVk, IC13x, IC13y, calldataload(add(pubSignals, 384)))
                g1_mulAccC(_pVk, IC14x, IC14y, calldataload(add(pubSignals, 416)))
                g1_mulAccC(_pVk, IC15x, IC15y, calldataload(add(pubSignals, 448)))
                g1_mulAccC(_pVk, IC16x, IC16y, calldataload(add(pubSignals, 480)))
                g1_mulAccC(_pVk, IC17x, IC17y, calldataload(add(pubSignals, 512)))
                g1_mulAccC(_pVk, IC18x, IC18y, calldataload(add(pubSignals, 544)))
                g1_mulAccC(_pVk, IC19x, IC19y, calldataload(add(pubSignals, 576)))
                g1_mulAccC(_pVk, IC20x, IC20y, calldataload(add(pubSignals, 608)))
                g1_mulAccC(_pVk, IC21x, IC21y, calldataload(add(pubSignals, 640)))
                g1_mulAccC(_pVk, IC22x, IC22y, calldataload(add(pubSignals, 672)))
                g1_mulAccC(_pVk, IC23x, IC23y, calldataload(add(pubSignals, 704)))
                g1_mulAccC(_pVk, IC24x, IC24y, calldataload(add(pubSignals, 736)))
                g1_mulAccC(_pVk, IC25x, IC25y, calldataload(add(pubSignals, 768)))
                g1_mulAccC(_pVk, IC26x, IC26y, calldataload(add(pubSignals, 800)))
                g1_mulAccC(_pVk, IC27x, IC27y, calldataload(add(pubSignals, 832)))
                g1_mulAccC(_pVk, IC28x, IC28y, calldataload(add(pubSignals, 864)))
                g1_mulAccC(_pVk, IC29x, IC29y, calldataload(add(pubSignals, 896)))
                g1_mulAccC(_pVk, IC30x, IC30y, calldataload(add(pubSignals, 928)))
                g1_mulAccC(_pVk, IC31x, IC31y, calldataload(add(pubSignals, 960)))
                g1_mulAccC(_pVk, IC32x, IC32y, calldataload(add(pubSignals, 992)))
                g1_mulAccC(_pVk, IC33x, IC33y, calldataload(add(pubSignals, 1024)))
                g1_mulAccC(_pVk, IC34x, IC34y, calldataload(add(pubSignals, 1056)))
                g1_mulAccC(_pVk, IC35x, IC35y, calldataload(add(pubSignals, 1088)))
                g1_mulAccC(_pVk, IC36x, IC36y, calldataload(add(pubSignals, 1120)))
                g1_mulAccC(_pVk, IC37x, IC37y, calldataload(add(pubSignals, 1152)))
                g1_mulAccC(_pVk, IC38x, IC38y, calldataload(add(pubSignals, 1184)))

                // -A
                mstore(_pPairing, calldataload(pA))
                mstore(add(_pPairing, 32), mod(sub(q, calldataload(add(pA, 32))), q))

                // B
                mstore(add(_pPairing, 64), calldataload(pB))
                mstore(add(_pPairing, 96), calldataload(add(pB, 32)))
                mstore(add(_pPairing, 128), calldataload(add(pB, 64)))
                mstore(add(_pPairing, 160), calldataload(add(pB, 96)))

                // alpha1
                mstore(add(_pPairing, 192), alphax)
                mstore(add(_pPairing, 224), alphay)

                // beta2
                mstore(add(_pPairing, 256), betax1)
                mstore(add(_pPairing, 288), betax2)
                mstore(add(_pPairing, 320), betay1)
                mstore(add(_pPairing, 352), betay2)

                // vk_x
                mstore(add(_pPairing, 384), mload(add(pMem, pVk)))
                mstore(add(_pPairing, 416), mload(add(pMem, add(pVk, 32))))


                // gamma2
                mstore(add(_pPairing, 448), gammax1)
                mstore(add(_pPairing, 480), gammax2)
                mstore(add(_pPairing, 512), gammay1)
                mstore(add(_pPairing, 544), gammay2)

                // C
                mstore(add(_pPairing, 576), calldataload(pC))
                mstore(add(_pPairing, 608), calldataload(add(pC, 32)))

                // delta2
                mstore(add(_pPairing, 640), deltax1)
                mstore(add(_pPairing, 672), deltax2)
                mstore(add(_pPairing, 704), deltay1)
                mstore(add(_pPairing, 736), deltay2)


                let success := staticcall(sub(gas(), 2000), 8, _pPairing, 768, _pPairing, 0x20)

                isOk := and(success, mload(_pPairing))
            }

            let pMem := mload(0x40)
            mstore(0x40, add(pMem, pLastMem))

            // Validate that all evaluations ∈ F
            
            checkField(calldataload(add(_pubSignals, 0)))
            
            checkField(calldataload(add(_pubSignals, 32)))
            
            checkField(calldataload(add(_pubSignals, 64)))
            
            checkField(calldataload(add(_pubSignals, 96)))
            
            checkField(calldataload(add(_pubSignals, 128)))
            
            checkField(calldataload(add(_pubSignals, 160)))
            
            checkField(calldataload(add(_pubSignals, 192)))
            
            checkField(calldataload(add(_pubSignals, 224)))
            
            checkField(calldataload(add(_pubSignals, 256)))
            
            checkField(calldataload(add(_pubSignals, 288)))
            
            checkField(calldataload(add(_pubSignals, 320)))
            
            checkField(calldataload(add(_pubSignals, 352)))
            
            checkField(calldataload(add(_pubSignals, 384)))
            
            checkField(calldataload(add(_pubSignals, 416)))
            
            checkField(calldataload(add(_pubSignals, 448)))
            
            checkField(calldataload(add(_pubSignals, 480)))
            
            checkField(calldataload(add(_pubSignals, 512)))
            
            checkField(calldataload(add(_pubSignals, 544)))
            
            checkField(calldataload(add(_pubSignals, 576)))
            
            checkField(calldataload(add(_pubSignals, 608)))
            
            checkField(calldataload(add(_pubSignals, 640)))
            
            checkField(calldataload(add(_pubSignals, 672)))
            
            checkField(calldataload(add(_pubSignals, 704)))
            
            checkField(calldataload(add(_pubSignals, 736)))
            
            checkField(calldataload(add(_pubSignals, 768)))
            
            checkField(calldataload(add(_pubSignals, 800)))
            
            checkField(calldataload(add(_pubSignals, 832)))
            
            checkField(calldataload(add(_pubSignals, 864)))
            
            checkField(calldataload(add(_pubSignals, 896)))
            
            checkField(calldataload(add(_pubSignals, 928)))
            
            checkField(calldataload(add(_pubSignals, 960)))
            
            checkField(calldataload(add(_pubSignals, 992)))
            
            checkField(calldataload(add(_pubSignals, 1024)))
            
            checkField(calldataload(add(_pubSignals, 1056)))
            
            checkField(calldataload(add(_pubSignals, 1088)))
            
            checkField(calldataload(add(_pubSignals, 1120)))
            
            checkField(calldataload(add(_pubSignals, 1152)))
            
            checkField(calldataload(add(_pubSignals, 1184)))
            
            checkField(calldataload(add(_pubSignals, 1216)))
            

            // Validate all evaluations
            let isValid := checkPairing(_pA, _pB, _pC, _pubSignals, pMem)

            mstore(0, isValid)
            
            return(0, 0x20)
        }
    }
}


/* =============================== */
/* Nova+CycleFold Decider verifier */
/**
 * @notice  Computes the decomposition of a `uint256` into num_limbs limbs of bits_per_limb bits each.
 * @dev     Compatible with sonobe::folding-schemes::folding::circuits::nonnative::nonnative_field_to_field_elements.
 */
library LimbsDecomposition {
    function decompose(uint256 x) internal pure returns (uint256[5] memory) {
        uint256[5] memory limbs;
        for (uint8 i = 0; i < 5; i++) {
            limbs[i] = (x >> (55 * i)) & ((1 << 55) - 1);
        }
        return limbs;
    }
}

/**
 * @author PSE & 0xPARC
 * @title  Interface for the NovaDecider contract hiding proof details.
 * @dev    This interface enables calling the verifyNovaProof function without exposing the proof details.
 */
interface OpaqueDecider {
    /**
     * @notice  Verifies a Nova+CycleFold proof given initial and final IVC states, number of steps and the rest proof inputs concatenated.
     * @dev     This function should simply reorganize arguments and pass them to the proper verification function.
     */
    function verifyOpaqueNovaProofWithInputs(
        uint256 steps, // number of folded steps (i)
        uint256[1] calldata initial_state, // initial IVC state (z0)
        uint256[1] calldata final_state, // IVC state after i steps (zi)
        uint256[25] calldata proof // the rest of the decider inputs
    ) external view returns (bool);

    /**
     * @notice  Verifies a Nova+CycleFold proof given all the proof inputs collected in a single array.
     * @dev     This function should simply reorganize arguments and pass them to the proper verification function.
     */
    function verifyOpaqueNovaProof(uint256[28] calldata proof) external view returns (bool);
}

/**
 * @author  PSE & 0xPARC
 * @title   NovaDecider contract, for verifying Nova IVC SNARK proofs.
 * @dev     This is an askama template which, when templated, features a Groth16 and KZG10 verifiers from which this contract inherits.
 */
contract NovaDecider is Groth16Verifier, KZG10Verifier, OpaqueDecider {
    /**
     * @notice  Computes the linear combination of a and b with r as the coefficient.
     * @dev     All ops are done mod the BN254 scalar field prime
     */
    function rlc(uint256 a, uint256 r, uint256 b) internal pure returns (uint256 result) {
        assembly {
            result := addmod(a, mulmod(r, b, BN254_SCALAR_FIELD), BN254_SCALAR_FIELD)
        }
    }

    /**
     * @notice  Verifies a nova cyclefold proof consisting of two KZG proofs and of a groth16 proof.
     * @dev     The selector of this function is "dynamic", since it depends on `z_len`.
     */
    function verifyNovaProof(
        // inputs are grouped to prevent errors due stack too deep
        uint256[3] calldata i_z0_zi, // [i, z0, zi] where |z0| == |zi|
        uint256[4] calldata U_i_cmW_U_i_cmE, // [U_i_cmW[2], U_i_cmE[2]]
        uint256[2] calldata u_i_cmW, // [u_i_cmW[2]]
        uint256[3] calldata cmT_r, // [cmT[2], r]
        uint256[2] calldata pA, // groth16 
        uint256[2][2] calldata pB, // groth16
        uint256[2] calldata pC, // groth16
        uint256[4] calldata challenge_W_challenge_E_kzg_evals, // [challenge_W, challenge_E, eval_W, eval_E]
        uint256[2][2] calldata kzg_proof // [proof_W, proof_E]
    ) public view returns (bool) {

        require(i_z0_zi[0] >= 2, "Folding: the number of folded steps should be at least 2");

        // from gamma_abc_len, we subtract 1. 
        uint256[38] memory public_inputs; 

        public_inputs[0] = 6697951387152592779041897379301182525256154873116403698283279190814149917880;
        public_inputs[1] = i_z0_zi[0];

        for (uint i = 0; i < 2; i++) {
            public_inputs[2 + i] = i_z0_zi[1 + i];
        }

        {
            // U_i.cmW + r * u_i.cmW
            uint256[2] memory mulScalarPoint = super.mulScalar([u_i_cmW[0], u_i_cmW[1]], cmT_r[2]);
            uint256[2] memory cmW = super.add([U_i_cmW_U_i_cmE[0], U_i_cmW_U_i_cmE[1]], mulScalarPoint);

            {
                uint256[5] memory cmW_x_limbs = LimbsDecomposition.decompose(cmW[0]);
                uint256[5] memory cmW_y_limbs = LimbsDecomposition.decompose(cmW[1]);
        
                for (uint8 k = 0; k < 5; k++) {
                    public_inputs[4 + k] = cmW_x_limbs[k];
                    public_inputs[9 + k] = cmW_y_limbs[k];
                }
            }
        
            require(this.check(cmW, kzg_proof[0], challenge_W_challenge_E_kzg_evals[0], challenge_W_challenge_E_kzg_evals[2]), "KZG: verifying proof for challenge W failed");
        }

        {
            // U_i.cmE + r * cmT
            uint256[2] memory mulScalarPoint = super.mulScalar([cmT_r[0], cmT_r[1]], cmT_r[2]);
            uint256[2] memory cmE = super.add([U_i_cmW_U_i_cmE[2], U_i_cmW_U_i_cmE[3]], mulScalarPoint);

            {
                uint256[5] memory cmE_x_limbs = LimbsDecomposition.decompose(cmE[0]);
                uint256[5] memory cmE_y_limbs = LimbsDecomposition.decompose(cmE[1]);
            
                for (uint8 k = 0; k < 5; k++) {
                    public_inputs[14 + k] = cmE_x_limbs[k];
                    public_inputs[19 + k] = cmE_y_limbs[k];
                }
            }

            require(this.check(cmE, kzg_proof[1], challenge_W_challenge_E_kzg_evals[1], challenge_W_challenge_E_kzg_evals[3]), "KZG: verifying proof for challenge E failed");
        }

        {
            // add challenges
            public_inputs[24] = challenge_W_challenge_E_kzg_evals[0];
            public_inputs[25] = challenge_W_challenge_E_kzg_evals[1];
            public_inputs[26] = challenge_W_challenge_E_kzg_evals[2];
            public_inputs[27] = challenge_W_challenge_E_kzg_evals[3];

            uint256[5] memory cmT_x_limbs;
            uint256[5] memory cmT_y_limbs;
        
            cmT_x_limbs = LimbsDecomposition.decompose(cmT_r[0]);
            cmT_y_limbs = LimbsDecomposition.decompose(cmT_r[1]);
        
            for (uint8 k = 0; k < 5; k++) {
                public_inputs[24 + 4 + k] = cmT_x_limbs[k]; 
                public_inputs[29 + 4 + k] = cmT_y_limbs[k];
            }

            bool success_g16 = this.verifyProof(pA, pB, pC, public_inputs);
            require(success_g16 == true, "Groth16: verifying proof failed");
        }

        return(true);
    }

    /**
     * @notice  Verifies a Nova+CycleFold proof given initial and final IVC states, number of steps and the rest proof inputs concatenated.
     * @dev     Simply reorganization of arguments and call to the `verifyNovaProof` function.
     */
    function verifyOpaqueNovaProofWithInputs(
        uint256 steps,
        uint256[1] calldata initial_state,
        uint256[1] calldata final_state,
        uint256[25] calldata proof
    ) public override view returns (bool) {
        uint256[1 + 2 * 1] memory i_z0_zi;
        i_z0_zi[0] = steps;
        for (uint256 i = 0; i < 1; i++) {
            i_z0_zi[i + 1] = initial_state[i];
            i_z0_zi[i + 1 + 1] = final_state[i];
        }

        uint256[4] memory U_i_cmW_U_i_cmE = [proof[0], proof[1], proof[2], proof[3]];
        uint256[2] memory u_i_cmW = [proof[4], proof[5]];
        uint256[3] memory cmT_r = [proof[6], proof[7], proof[8]];
        uint256[2] memory pA = [proof[9], proof[10]];
        uint256[2][2] memory pB = [[proof[11], proof[12]], [proof[13], proof[14]]];
        uint256[2] memory pC = [proof[15], proof[16]];
        uint256[4] memory challenge_W_challenge_E_kzg_evals = [proof[17], proof[18], proof[19], proof[20]];
        uint256[2][2] memory kzg_proof = [[proof[21], proof[22]], [proof[23], proof[24]]];

        return this.verifyNovaProof(
            i_z0_zi,
            U_i_cmW_U_i_cmE,
            u_i_cmW,
            cmT_r,
            pA,
            pB,
            pC,
            challenge_W_challenge_E_kzg_evals,
            kzg_proof
        );
    }

    /**
     * @notice  Verifies a Nova+CycleFold proof given all proof inputs concatenated.
     * @dev     Simply reorganization of arguments and call to the `verifyNovaProof` function.
     */
    function verifyOpaqueNovaProof(uint256[28] calldata proof) public override view returns (bool) {
        uint256[1] memory z0;
        uint256[1] memory zi;
        for (uint256 i = 0; i < 1; i++) {
            z0[i] = proof[i + 1];
            zi[i] = proof[i + 1 + 1];
        }

        uint256[25] memory extracted_proof;
        for (uint256 i = 0; i < 25; i++) {
            extracted_proof[i] = proof[3 + i];
        }

        return this.verifyOpaqueNovaProofWithInputs(proof[0], z0, zi, extracted_proof);
    }
}