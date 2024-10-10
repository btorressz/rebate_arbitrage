// No imports needed: web3, anchor, pg and more are globally available
//THIS TEST FILE IS ONLY FOR ME TO USE

describe("Rebate Arbitrage Tests", () => {
  let poolKp: web3.Keypair;
  let userKp: web3.Keypair;
  let stakedLiquidityKp: web3.Keypair;
  let tradeHistoryKp: web3.Keypair;

  before(async () => {
    // Generate keypairs for the pool and user accounts
    poolKp = new web3.Keypair();
    userKp = new web3.Keypair();
    stakedLiquidityKp = new web3.Keypair();
    tradeHistoryKp = new web3.Keypair();

    // Airdrop SOL to the user to cover transaction fees
    const airdropSig = await pg.connection.requestAirdrop(
      userKp.publicKey,
      web3.LAMPORTS_PER_SOL
    );
    await pg.connection.confirmTransaction(airdropSig);
  });

  it("initializes the pool", async () => {
    const feeRate = new BN(100); // Example fee rate (1%)
    const tokenAMint = new web3.PublicKey("So11111111111111111111111111111111111111112");
    const tokenBMint = new web3.PublicKey("So11111111111111111111111111111111111111113");

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

    const poolAccount = await pg.program.account.pool.fetch(poolKp.publicKey);
    console.log("Initialized Pool:", poolAccount);
  });

  it("initializes the user", async () => {
    const initialBalance = new BN(1000); // Example initial balance

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

    const userAccount = await pg.program.account.user.fetch(userKp.publicKey);
    console.log("Initialized User:", userAccount);
    assert(userAccount.tokenABalance.eq(initialBalance));
  });

  it("provides liquidity", async () => {
    const amount = new BN(500); // Example amount of liquidity to provide

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

    const poolAccount = await pg.program.account.pool.fetch(poolKp.publicKey);
    console.log("Pool after providing liquidity:", poolAccount);
    assert(poolAccount.liquidity.eq(amount));
  });

  it("stakes liquidity with lock", async () => {
    const amount = new BN(200); // Example amount to stake
    const lockDuration = new BN(30 * 24 * 60 * 60); // 30 days in seconds

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

    const stakedLiquidityAccount = await pg.program.account.stakedLiquidity.fetch(
      stakedLiquidityKp.publicKey
    );
    console.log("Staked Liquidity:", stakedLiquidityAccount);
    assert(stakedLiquidityAccount.amount.eq(amount));
  });

  it("trades with slippage", async () => {
    const tradeAmount = new BN(100); // Example trade amount

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

    const userAccount = await pg.program.account.user.fetch(userKp.publicKey);
    console.log("User after trading:", userAccount);

    const tradeHistoryAccount = await pg.program.account.tradeHistory.fetch(
      tradeHistoryKp.publicKey
    );
    console.log("Trade History:", tradeHistoryAccount);
    assert(tradeHistoryAccount.tradeAmount.eq(tradeAmount));
  });
});
