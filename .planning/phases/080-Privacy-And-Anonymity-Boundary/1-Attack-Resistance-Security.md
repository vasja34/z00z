# 11. Attack Resistance & Security

## 11.1 Anti-Burn Attacks

### 11.1.1 Burn Attack Vector

**TBD** - Sender creates invalid enc_pack

### 11.1.2 OWF Defense

**TBD** - Proof of correct encryption

#### 11.1.2.1 enc_pack Authenticity

**TBD** - AEAD tag binding

#### 11.1.2.2 Pedersen Opening Guarantee

**TBD** - Receiver can always open C_amount

### 11.1.3 M1 Check (owner_tag_expected)

**TBD** - Early rejection before decrypt

## 11.2 Anti-Theft Attacks

### 11.2.1 Sender Theft Vector

**TBD** - Sender knows all output data

### 11.2.2 Spend Proof Defense

**TBD** - Requires receiver_secret

#### 11.2.2.1 Ownership Constraints

**TBD** - Derive owner_handle from receiver_secret

#### 11.2.2.2 Why Sender Cannot Spend

**TBD** - Lacks receiver_secret

## 11.3 Anti-Malleability Attacks

### 11.3.1 Malleability Vector

**TBD** - Rebind enc_pack to different leaf

### 11.3.2 leaf_ad Defense

**TBD** - Associated data binds to leaf fields

#### 11.3.2.1 AEAD with AD

**TBD** - Tag fails on rebind

## 11.4 Anti-DoS Attacks

### 11.4.1 Targeted Tag Spam

**TBD** - Attacker generates false tag16 matches

### 11.4.2 Mitigation: req_id in tag16

**TBD** - tag16 depends on secret req_id

#### 11.4.2.1 Attacker Without req_id

**TBD** - Cannot target specific receiver

### 11.4.3 Wallet-Side Mitigation

**TBD** - Rate limiting, background scan

#### 11.4.3.1 Rate Limit

**TBD** - Limit decryption attempts

#### 11.4.3.2 Background Scan

**TBD** - Don't freeze UI

#### 11.4.3.3 Fetch-on-Demand

**TBD** - Lazy leaf body fetching

## 11.5 Directory/Relay Attacks

### 11.5.1 Malicious Directory

**TBD** - Provides wrong view_pk

### 11.5.2 Identity Pinning Defense (Variant 1)

**TBD** - TOFU + pin storage

#### 11.5.2.1 First-Use Pin

**TBD** - Cache (owner_handle → view_pk)

#### 11.5.2.2 Rotation Warning

**TBD** - Alert on view_pk change

### 11.5.3 PaymentRequest Defense

**TBD** - Signed request with identity_pk

#### 11.5.3.1 Signature Verification

**TBD** - Check identity_pk signature

#### 11.5.3.2 Identity Pinning

**TBD** - Verify against known identity_pk

## 11.6 Refund Paths (Optional)

### 11.6.1 Refund Mechanism

**TBD** - Timelock-based reclaim

#### 11.6.1.1 Receiver Claim

**TBD** - Immediate claim by receiver

#### 11.6.1.2 Sender Reclaim

**TBD** - After timeout T

### 11.6.2 Privacy Impact

**TBD** - Claim/reclaim may reveal info