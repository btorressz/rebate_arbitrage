// Client
console.log("My address:", pg.wallet.publicKey.toString());

// Check the balance of the wallet
const balance = await pg.connection.getBalance(pg.wallet.publicKey);
console.log(`My balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);

// Generate keypairs for accounts
const poolKp = new web3.Keypair();
const userKp = new web3.Keypair();
const stakedLiquidityKp = new web3.Keypair();
const tradeHistoryKp = new web3.Keypair();

// Airdrop SOL to the new user account for testing
console.log("Requesting airdrop...");
await pg.connection.requestAirdrop(userKp.publicKey, 2 * web3.LAMPORTS_PER_SOL);
console.log("Airdrop completed.");

// Initialize Pool
async function initializePool() {
  const feeRate = new BN(100); // Example fee rate (1%)
  const tokenAMint = new web3.PublicKey("So11111111111111111111111111111111111111112");
  const tokenBMint = new web3.PublicKey("So11111111111111111111111111111111111111113");

  console.log("Initializing pool...");
  const txHash = await pg.program.methods
    .initializePool(feeRate, tokenAMint, tokenBMint)
    .accounts({
      pool: poolKp.publicKey,
      userSigner: pg.wallet.publicKey,
      systemProgram: web3.SystemProgram.programId,
    })
    .signers([poolKp])
    .rpc();

  console.log(`Initialize Pool Tx: ${txHash}`);
  await pg.connection.confirmTransaction(txHash);
  
  // Fetch and display pool data
  const poolAccount = await pg.program.account.pool.fetch(poolKp.publicKey);
  console.log("Initialized Pool Data:", poolAccount);
}

// Initialize User
async function initializeUser() {
  const initialBalance = new BN(1000); // Example initial balance

  console.log("Initializing user...");
  const txHash = await pg.program.methods
    .initializeUser(initialBalance)
    .accounts({
      user: userKp.publicKey,
      userSigner: pg.wallet.publicKey,
      systemProgram: web3.SystemProgram.programId,
    })
    .signers([userKp])
    .rpc();

  console.log(`Initialize User Tx: ${txHash}`);
  await pg.connection.confirmTransaction(txHash);

  // Fetch and display user data
  const userAccount = await pg.program.account.user.fetch(userKp.publicKey);
  console.log("Initialized User Data:", userAccount);
}

// Provide Liquidity
async function provideLiquidity() {
  const amount = new BN(500); // Example liquidity amount

  console.log("Providing liquidity...");
  const txHash = await pg.program.methods
    .provideLiquidity(amount)
    .accounts({
      pool: poolKp.publicKey,
      user: userKp.publicKey,
    })
    .signers([userKp])
    .rpc();

  console.log(`Provide Liquidity Tx: ${txHash}`);
  await pg.connection.confirmTransaction(txHash);

  // Fetch and display pool data
  const poolAccount = await pg.program.account.pool.fetch(poolKp.publicKey);
  console.log("Pool Data After Providing Liquidity:", poolAccount);
}

// Stake Liquidity with Lock
async function stakeLiquidityWithLock() {
  const amount = new BN(200); // Example amount to stake
  const lockDuration = new BN(30 * 24 * 60 * 60); // 30 days in seconds

  console.log("Staking liquidity with lock...");
  const txHash = await pg.program.methods
    .stakeLiquidityWithLock(amount, lockDuration)
    .accounts({
      pool: poolKp.publicKey,
      user: userKp.publicKey,
      stakedLiquidity: stakedLiquidityKp.publicKey,
      systemProgram: web3.SystemProgram.programId,
    })
    .signers([userKp, stakedLiquidityKp])
    .rpc();

  console.log(`Stake Liquidity with Lock Tx: ${txHash}`);
  await pg.connection.confirmTransaction(txHash);

  // Fetch and display staked liquidity data
  const stakedLiquidityAccount = await pg.program.account.stakedLiquidity.fetch(
    stakedLiquidityKp.publicKey
  );
  console.log("Staked Liquidity Data:", stakedLiquidityAccount);
}

// Trade with Slippage
async function tradeWithSlippage() {
  const tradeAmount = new BN(100); // Example trade amount

  console.log("Trading with slippage...");
  const txHash = await pg.program.methods
    .tradeWithSlippage(tradeAmount)
    .accounts({
      pool: poolKp.publicKey,
      user: userKp.publicKey,
      tradeHistory: tradeHistoryKp.publicKey,
      systemProgram: web3.SystemProgram.programId,
    })
    .signers([userKp, tradeHistoryKp])
    .rpc();

  console.log(`Trade with Slippage Tx: ${txHash}`);
  await pg.connection.confirmTransaction(txHash);

  // Fetch and display user data
  const userAccount = await pg.program.account.user.fetch(userKp.publicKey);
  console.log("User Data After Trading:", userAccount);

  // Fetch and display trade history data
  const tradeHistoryAccount = await pg.program.account.tradeHistory.fetch(
    tradeHistoryKp.publicKey
  );
  console.log("Trade History Data:", tradeHistoryAccount);
}

// Run the functions sequentially
(async () => {
  try {
    await initializePool();
    await initializeUser();
    await provideLiquidity();
    await stakeLiquidityWithLock();
    await tradeWithSlippage();
  } catch (err) {
    console.error("An error occurred:", err);
  }
})();
