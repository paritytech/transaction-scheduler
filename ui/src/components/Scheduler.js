import React, { Component } from 'react'
import { Header, Button, Form, Label } from 'semantic-ui-react'
import DatePicker from 'react-datepicker'
import numeral from 'numeral'
import moment from 'moment'

import 'react-datepicker/dist/react-datepicker.css'
import './Scheduler.css'

const dateFormat = {
  sameElse: 'llll'
}

export function Summary({ condition }) {
    if ('block' in condition) {
      const block = parseInt(condition.block.substr(2), 16)
      return (
        <span>at block #{ numeral(block).format() }</span>
      )
    }

    if ('time' in condition) {
      return (
        <span>{ moment.unix(condition.time).calendar(null, dateFormat) }</span>
      )
    }
}

export default class Scheduler extends Component {
  static defaultProps = {
    onNewCondition: () => {},
    currentBlock: 0,
  }

  state = {
    mode: 'time',
    inputBlock: '',
    parseBlock: 0,
    minBlock: this.props.currentBlock,

    fineTune: false,
    tempTime: moment().add(3, 'hours').unix(),
    tempTimeError: false,

    startTime: moment(),
    inputTime: moment().add(3, 'hours')
  }

  setModeTime = () => this.setState({ mode: 'time'})
  setModeBlock = () => this.setState({ mode: 'block' })

  isTimeValid (inputTime = this.state.inputTime) {
    return inputTime > moment()
  }

  componentDidMount () {
    this.props.onNewCondition({ time: this.state.inputTime.unix() })
  }

  inputTime (momentTime) {
    return {
      inputTime: momentTime,
      tempTime: momentTime.unix(),
      tempTimeError: false
    }
  }

  componentWillReceiveProps (newProps) {
    const { condition } = newProps
    if (this.props.condition === condition) {
      return
    }

    if ('time' in condition) {
      this.setState({
        mode: 'time',
        ...this.inputTime(moment.unix(condition.time))
      })
    }

    if ('block' in condition) {
      this.setState({
        mode: 'block',
        ...this.parseBlock(condition.block)
      })
    }
  }

  render () {
    const { mode } = this.state
    return (
      <div style={styles.scheduler}>
        <Header as='h3'>I want my transaction to run at specific:</Header>
        <Button.Group attached widths={2}>
          <Button
            active={mode === 'time' }
            onClick={ this.setModeTime }
            primary={ mode === 'time' }
          >time</Button>
          <Button.Or color='purple' />
          <Button
            active={mode === 'block' }
            onClick={ this.setModeBlock }
            primary={ mode === 'block' }
          >block</Button>
        </Button.Group>

        {mode === 'time' ? this.renderTimeSelector() : null }
        {mode === 'block' ? this.renderBlockSelector() : null }

        <div style={{marginTop: '1rem'}}>
          { this.renderSummary() }
        </div>
      </div>
    )
  }

  handleInputTime = inputTime => {
    this.setState(this.inputTime(inputTime))
    if (this.isTimeValid(inputTime)) {
      this.props.onNewCondition({ time: inputTime.unix() })
    }
  }

  secondsToMoment = seconds => moment(parseInt(seconds, 10) * 1000)

  handleInputRawTime = () => {
    const { tempTime } = this.state
    this.handleInputTime(this.secondsToMoment(tempTime))
  }

  handleInputRawTimeTemp = ev => {
    const { value } = ev.target
    const v = this.secondsToMoment(value)
    this.setState({
      tempTime: value,
      tempTimeError: !this.isTimeValid(v)
    })
  }

  handleFineTune = () => {
    this.setState({ fineTune: !this.state.fineTune })
  }

  renderTimeSelector () {
    const { inputTime, startTime, fineTune, tempTime, tempTimeError } = this.state
    const tempMoment = this.secondsToMoment(tempTime)

    return (
      <Form as='div'>
        <Form.Field>
          <DatePicker
            inline
            onChange={ this.handleInputTime }
            selected={ inputTime }
            showTimeSelect
            minDate={ startTime }
          />
          { this.renderTimeHelp() }
          <a
            className='finetune'
            onClick={this.handleFineTune}
          >fine-tune</a>
        </Form.Field>
        { fineTune && (
          <Form.Group inline widths='two'>
            <Form.Input
              error={ tempTimeError }
              fluid
              onChange={ this.handleInputRawTimeTemp }
              size='mini'
              type='number'
              value={ tempTime }
            />
            <Form.Button
              disabled={ tempTimeError || tempMoment === inputTime }
              fluid
              onClick={ this.handleInputRawTime }
              primary
              size='mini'
              content='Update Time'
            />
          </Form.Group>
        )}
      </Form>
    )
  }

  renderTimeHelp () {
    if (!this.isTimeValid()) {
      return (
        <Label pointing basic color='red'>You need to select a future time.</Label>
      )
    }

    return null
  }

  parseBlock (inputBlock) {
    const minBlock = this.props.currentBlock
    const parsedBlock = inputBlock.startsWith('0x')
      ? parseInt(inputBlock.substr(2), 16)
      : numeral(inputBlock).value()
    const validBlock = parsedBlock > minBlock

    return { inputBlock, validBlock, minBlock, parsedBlock }
  }

  handleInputBlock = (ev) => {
    const state = this.parseBlock(ev.target.value)
    this.setState(state)

    if (state.validBlock) {
      this.props.onNewCondition({
        block: '0x' + state.parsedBlock.toString(16)
      })
    }
  }

  renderBlockSelector () {
    const { inputBlock } = this.state
    return (
      <Form as='div'>
        <Form.Field>
          <input
            type='text'
            placeholder={ 'enter block number' }
            value={ inputBlock }
            onChange={ this.handleInputBlock }
          />
          { this.renderBlockHelp() }
        </Form.Field>
      </Form>
    )
  }

  renderBlockHelp () {
    const { minBlock, validBlock } = this.state
    const { currentBlock } = this.props
    if (!validBlock) {
      return (
        <Label pointing basic color='red'>Number needs to be greater than { numeral(minBlock).format() }</Label>
      )
    }

    if (currentBlock) {
      return (
        <Label pointing>Current block: <strong>#{ numeral(currentBlock).format() }</strong></Label>
      )
    }

    return null
  }

  renderSummary () {
    const { parsedBlock, mode, validBlock, inputTime } = this.state

    if (mode === 'block' && validBlock) {
      return (
        <p>Your transaction will be propagated to the network at block #{ numeral(parsedBlock).format() }.</p>
      )
    }

    if (mode === 'time' && this.isTimeValid()) {
      return (
        <p>Your transaction will be propagted to the network { inputTime.calendar(null, dateFormat) } ({ moment(inputTime).fromNow() })</p>
      )
    }
  }
}

const styles = {
  scheduler: {
    width: '100%',
    maxWidth: '350px',
    margin: 'auto',
    textAlign: 'center'
  }
}
