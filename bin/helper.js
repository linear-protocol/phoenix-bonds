function getEnvConfig(env) {
  return require(`./env/${env}/config.js`);
}

function daysToMs(n) {
  return parseInt(n * 24 * 3600 * 1000);
}

function hoursToMs(n) {
  return parseInt(n * 3600 * 1000);
}

/**
 * 
 * @param {nearAPI.Near} near 
 * @param {string} signer 
 * @param {string} tokenId 
 * @param {string} receiverId 
 */
async function storageDeposit (signer, tokenId, receiverId) {
  const storageBalance = await signer.viewFunction(
    tokenId,
    'storage_balance_of',
    {
      account_id: receiverId
    }
  )
  if (!storageBalance) {
    await signer.functionCall({
      contractId: tokenId,
      methodName: 'storage_deposit',
      args: {
        account_id: receiverId
      },
      attachedDeposit: parseNearAmount('0.1')
    });
    console.log(`FT ${tokenId} storage deposited for ${receiverId}`);
  } else {
    console.log(`FT ${tokenId} is already registered for ${receiverId}`)
  }
}


module.exports = {
  getEnvConfig,
  daysToMs,
  hoursToMs,
  storageDeposit,
}
