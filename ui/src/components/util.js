import Eth from 'ethjs'

export function toChecksumAddress (address) {
  address = address.replace('0x', '').toLowerCase()
  const addressHash = Eth.keccak256(address)

  return '0x' + Array(40).join('.').split('.').map((x, i) => {
    const c = address[i] || ''
    if (parseInt(addressHash[i], 16) > 7) {
      return c.toUpperCase()
    }
    return c
  }).join('')
}
