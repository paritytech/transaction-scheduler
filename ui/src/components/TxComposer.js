import React, { Component } from 'react'
import { Header, Label, Button, Form, Select, Divider, Message, Grid } from 'semantic-ui-react'

import Eth from 'ethjs'
import numeral from 'numeral'

import { toChecksumAddress } from './util'

const CONTRACT_WARNING = 'You didn\'t specify recipient - this will create a contract'

export default class TxComposer extends Component {
  static defaultProps = {
    gasPriceOptions: [],
    accounts: [],
    onAddGasPrice: () => {},
    displayButton: false
  }

  state = {
    sending: false,
    result: {},
    sender: '',
    senderWarning: CONTRACT_WARNING,
    gasPrice: '0',
    value: '',
    numValue: Eth.toBN(0),
    gasLimit: '21k',
    numGasLimit: Eth.toBN(21000),
    data: '',
    recipient: '',
    recipientWarning: CONTRACT_WARNING
  }

  componentDidMount () {
    this.updateSender(this.props.accounts)
    this.updateGasPrice(this.props.gasPriceOptions)
  }

  componentWillReceiveProps (newProps) {
    if (newProps.accounts !== this.props.accounts) {
      this.updateSender(newProps.accounts)
    }
    if (newProps.gasPriceOptions !== this.props.gasPriceOptions) {
      this.updateGasPrice(newProps.gasPriceOptions)
    }
  }

  updateGasPrice (options) {
    if (this.state.gasPrice !== '0' || !options.length) {
      return
    }

    this.setState({
      gasPrice: options[options.length / 2].value
    })
  }

  updateSender (accounts) {
    const {
      value: sender,
      warning: senderWarning
    } = this.handleAddress(accounts.length ? accounts[0].value : '')
    
    this.setState({
      sender, senderWarning
    })
  }
  
  handleGasPrice = (ev, gasPrice) => {
    const { gasPriceOptions } = this.props
    const value = parseValue(gasPrice.value).toString()

    if (!gasPriceOptions.find(x => x.value === value)) {
      this.props.onAddGasPrice({
        value,
        text: gasPrice.value
      })
    }

    this.setState({
      gasPrice: value,
    })
  }

  handleValue = (ev) => {
    const { value } = ev.target
    const numValue = parseValue(value)
    this.setState({
      value,
      numValue
    })
  }

  handleGasLimit = (ev) => {
    const { value } = ev.target
    const numValue = parseValue(value)
    this.setState({
      gasLimit: value,
      numGasLimit: numValue
    })
  }

  handleAddress = (value) => {
    let warning = ''
    if (!value) {
      warning = CONTRACT_WARNING
    }

    if (!value.startsWith('0x') || value.length !== 42) {
      warning = 'This does not look like Ethereum address.'
    }

    if (
      !warning
      && value.toLowerCase() !== value
      && value !== toChecksumAddress(value)
    )  {
      warning = 'The address does not contain a valid checksum.'
    }

    return {
      value, warning
    }
  }

  handleRecipient = ev => {
    const {
      value: recipient,
      warning: recipientWarning
    } = this.handleAddress(ev.target.value)

    this.setState({
      recipient, recipientWarning
    })
  }

  handleSenderText = ev => {
    const {
      value: sender,
      warning: senderWarning
    } = this.handleAddress(ev.target.value)
    
    this.setState({
      sender, senderWarning
    })
  }

  handleSender = (ev, data) => {
    this.setState({
      sender: data.value,
      senderWarning: ''
    })
  }

  handleCreateTransaction = ev => {
    this.setState({
      sending: true
    })

    this.props.onSend(this.json())
      .then(tx => {
        this.setState({
          sending: false,
          result: { success: tx }
        })
      })
      .catch(err => {
        this.setState({
          sending: false,
          result: { error: err }
        })
      })
  }

  json () {
    const { sender, data, numGasLimit, numValue, gasPrice, recipient } = this.state

    return {
        from: sender,
        to: recipient ? recipient : null,
        value: '0x' + numValue.toString(16),
        gasPrice: '0x' + Eth.toBN(gasPrice).toString(16),
        gas: '0x' + numGasLimit.toString(16),
        data: data ? data : null
      }
  }

  handleData = ev => {
    const { value } = ev.target
    const dataError = isDataValid(value) ? '' : 'This does not look like valid data.'
    
    this.setState({
      data: value,
      dataError
    })
  }

  render () {
    const { 
      sending, result,
      sender, senderWarning,
      data, dataError,
      gasPrice,
      value, numValue,
      gasLimit, numGasLimit,
      recipient, recipientWarning
    } = this.state

    const {
      accounts,
      displayButton,
      gasPriceOptions
    } = this.props

    return (
      <div>
        <Form
          loading={sending}
          success={!!result.success}
          error={!!result.error}
        >
          <Grid doubling>
            <Grid.Column width={8}>
              {
                accounts.length
                ? (
                  <Form.Field error={!sender}>
                    <label>Sender (your) Address</label>
                    <Select
                      options={accounts}
                      placeholder='From'
                      value={sender}
                      onChange={this.handleSender}
                    />
                    <Label pointing basic>You need a private key for that account.</Label>
                  </Form.Field>
                )
                : (
                  <Form.Field>
                    <label>Sender (your) Address</label>
                    <input
                      type='text'
                      placeholder='From'
                      recipient={sender}
                      onChange={this.handleSenderText}
                    />
                    { !senderWarning
                        ? <Label pointing basic>You need a private key for that account.</Label>
                        : <Label pointing basic color='yellow'>{senderWarning}</Label>
                    }
                  </Form.Field>
                )
              }
              <Form.Field>
                <label>Recipient Address</label>
                <input
                  type='text'
                  placeholder='To'
                  recipient={recipient}
                  onChange={this.handleRecipient}
                />
                { !recipientWarning
                  ? null
                  : <Label pointing basic color='yellow'>{recipientWarning}</Label>
                }
              </Form.Field>
              <Form.Field>
                <label>Transferred Value</label>
                <input
                  type='text'
                  placeholder='value'
                  value={value}
                  onChange={this.handleValue}
                />
                <Label pointing basic>
                  { numValue.isZero()
                      ? <span>Try typing: <code>1k shannon</code> or <code>0x123</code> here.</span>
                      : <span>{ numeral(numValue.toString()).format() } wei (0x{ numValue.toString(16) })</span>
                  }
                </Label>
              </Form.Field>
            </Grid.Column>
            <Grid.Column width={8}>
              <Form.Field>
                <label>Data</label>
                <input
                  type='text'
                  placeholder='value'
                  value={data}
                  onChange={this.handleData}
                />
                <Label pointing basic color={dataError ? 'red' : null}>
                  { !dataError
                      ? <span>Hex-encoded data you want to attach to the transaction</span>
                      : <span>{ dataError }</span>
                  }
                </Label>
              </Form.Field>
              <Form.Field>
                <label>Gas Price</label>
                { gasPriceOptions.length
                  ? (
                    <Select
                      allowAdditions
                      onChange={this.handleGasPrice}
                      options={gasPriceOptions}
                      search
                      placeholder='Gas Price'
                      value={gasPrice}
                    />
                  ) : null
                }
                { Eth.toBN(gasPrice).isZero()
                      ? null 
                      : <Label pointing basic>
                        <span>{ numeral(gasPrice).format() } wei (0x{ Eth.toBN(gasPrice).toString(16) })</span>
                      </Label>
                  }
              </Form.Field>
              <Form.Field>
                <label>Gas (Gas Limit)</label>
                <input
                  type='text'
                  placeholder='value'
                  value={gasLimit}
                  onChange={this.handleGasLimit}
                />
                <Label pointing basic>
                  { numGasLimit.isZero()
                      ? <span>Try typing: <code>21k</code> or <code>0x123</code> here.</span>
                      : <span>{ numeral(numGasLimit.toString()).format() } gas (0x{ numGasLimit.toString(16) })</span>
                  }
                </Label>
              </Form.Field>
            </Grid.Column>
          </Grid>

          <Divider hidden />

          { displayButton 
              ? (
                <Button
                  primary
                  type='submit'
                  onClick={this.handleCreateTransaction}
                >
                  Create Transaction<br />
                </Button>
              )
              : null
          }

          <Divider hidden />

          { this.renderResult() }
        </Form>

        <Divider hidden />
        <Divider horizontal>{ displayButton ? 'or' : 'and' }</Divider>
        <Divider hidden />
        <TxComposerCurl json={this.json()} />
      </div>
    )
  }

  renderResult () {
    const { result } = this.state

    if (result.success) {
      return (
        <Message success>
          <Message.Header>RAW transaction created</Message.Header>
          <code>{ result.success.tx.raw }</code>
          <p>You can now proceed to step 2 and schedule the RAW transaction.</p>
        </Message>
      )
    }

    if (result.error) {
      return (
        <Message error>
          <Message.Header>RAW transaction failed</Message.Header>
          <code>{ result.error.toString() }</code>
        </Message>
      )
    }
  }
}

function isDataValid(data) {
  if (!data) {
    return true
  }

  return data.startsWith('0x') && data.length % 2 === 0 && !data.substr(2).split('').find(d => {
    return window.isNaN(parseInt(d, 16))
  })
}

function replacePrefixes(num) {
  return num
    .replace(/k/g, '000')
    .replace(/m/g, '000000')
    .replace(/g/g, '000000000')
}

function parseValue(v) {
  v = v.toLowerCase()

  if (v.startsWith('0x')) {
    return new Eth.BN(v.substr(2), 16)
  }

  const parts = v.replace(/^\s*/, '').replace(/\s*$/, '').split(/\s+/)
  if (parts.length > 1) {
    try {
      return Eth.toWei(replacePrefixes(parts[0]), parts[1])
    } catch (e) {}
  }

  const num = parseInt(replacePrefixes(v), 10)
  if (!window.isNaN(num)) {
    return Eth.toBN(num)
  }

  return Eth.toBN(0)
}


function TxComposerCurl({ json }) {
  const request = {
    jsonrpc: '2.0',
    id: 1,
    method: 'eth_signTransaction',
    params: [json]
  }

  return (
    <div>
      <Header as='h3'>Send CURL Request</Header>
      <p>If you don't have web3-enabled browser you can send this RPC request to your node.</p>
      <div className='code'>
        $ curl localhost:8545 -X POST -HContent-Type:application/json --data '{ JSON.stringify(request) }'
      </div>
    </div>
  )
}
