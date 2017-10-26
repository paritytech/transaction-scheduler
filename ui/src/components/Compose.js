import React, { Component } from 'react'
import { Container, Header, Label, Message, Button, Divider } from 'semantic-ui-react'
import Eth from 'ethjs'
import EthRPC from 'ethjs-rpc'

import TxComposer from './TxComposer'
import { toChecksumAddress } from './util'

export default class Compose extends Component {
  state = {
    accounts: [],
    gasPriceOptions: Array(40).join('.').split('.').map((x, idx) => {
      const id = idx + 1

      return {
        key: id,
        value: Eth.toBN(id * 1e9).toString(),
        text: `${id} gwei`
      }
    })
  }

  render () {
    const isWeb3 = this.web3Detected()

    return (
      <Container text style={{ marginTop: '2rem' }}>
        <Header as='h1'>
          <Label circular color='purple' size='massive'>1</Label>
          <a name='compose'>Create Transaction</a>
        </Header>
        <p>Skip this step if you already have RAW signed transaction RLP.</p>
        <Divider hidden />

        { isWeb3
            ? this.renderComposer()
            : this.renderNoWeb3()
        }
        <Divider hidden />
        { this.renderSignDocs(isWeb3) }
        <Divider hidden />
      </Container>
    )
  }

  web3Detected () {
    return typeof window.web3 !== 'undefined' && typeof window.web3.currentProvider !== 'undefined'
  }

  renderComposer () {
    return (
      <EthComposer
        onRlpChange={this.props.onRlpChange}
      />
    )
  }

  renderNoWeb3 () {
    const { accounts, gasPriceOptions } = this.state

    return (
      <div>
        <Message
          error
          style={{ textAlign: 'center' }}
        >
          <Message.Header>
            Web3-enabled browser required
          </Message.Header>
          <p>
            To create transaction on this page you need to be running a web3-enabled browser.
          </p>

          <Button 
            as='a'
            href="https://chrome.google.com/webstore/detail/parity-ethereum-integrati/himekenlppkgeaoeddcliojfddemadig"
            primary
          >
            Install Parity Chrome Extension
          </Button>
        </Message>
        <TxComposer
          accounts={accounts}
          gasPriceOptions={gasPriceOptions}
          onAddGasPrice={this.handleAddGasPrice}
        />
      </div>
    )
  }

  renderSignDocs (isWeb3) {
    return null
  }
}

class EthComposer extends Component {
  state = {
    accounts: [],
    gasPriceOptions: []
  }

  componentDidMount() {
    const poll = (call, state, time = 2500) => {
      let last = null
      const check = () => {
        call().then(x => {
          const str = JSON.stringify(x)
          if (str === last) {
            return
          }
          last = str
          this.setState(state(x))
        })
      };
      check()
      const int = window.setInterval(check, time)
      return () => {
        window.clearInterval(int)
      }
    }

    this.eth = new Eth(window.web3.currentProvider)
    this.rpc = new EthRPC(window.web3.currentProvider)
    this.gasPrice = poll(() => this.eth.gasPrice(), gasPrice => {
      const gwei = Eth.toBN(1e9)
      const options = Array(gasPrice.div(gwei).mul(Eth.toBN(2)).toNumber()).join('.').split('.').map((x, idx) => ({
        key: idx,
        value: Eth.toBN(idx + 1).mul(gwei).toString(),
        text: `${idx + 1} gwei`
      }))

      return {
        gasPriceOptions: options
      }
    }, 2500)
    this.accounts = poll(() => this.eth.accounts(), accounts => {
      const vals = accounts.map(toChecksumAddress).map(account => ({
        key: account,
        value: account,
        text: account.substr(0, 12) + '...' + account.substr(32)
      })); 
      return {
        sender: vals[0] ? vals[0].value : null,
        accounts: vals
      }
    }, 5000)
  }

  componentWillUnmount() {
    this.accounts()
    this.gasPrice()
    this.eth = null
    this.rpc = null
  }

  handleAddGasPrice = data => {
    const { gasPriceOptions } = this.state
    this.setState({
      gasPriceOptions: gasPriceOptions.concat(data)
    })
  }

  handleSend = json => {
    return new Promise((resolve, reject) => {
      this.rpc.sendAsync({
          method: 'eth_signTransaction',
          params: [json]
      }, (err, res) => err ? reject(err) : resolve(res))
    }).then(tx => {
      debugger;
      this.props.onRlpChange(tx.raw)
      return tx
    })
  }

  render () {
    const { accounts, gasPriceOptions } = this.state

    return (
      <TxComposer
        accounts={accounts}
        displayButton
        gasPriceOptions={gasPriceOptions}
        onAddGasPrice={this.handleAddGasPrice}
        onSend={this.handleSend}
      />
    )
  }
}
