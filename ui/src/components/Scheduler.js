import React, { Component } from 'react'
import { Header, Button, Form, Label } from 'semantic-ui-react'
import DatePicker from 'react-datepicker'
import numeral from 'numeral'
import moment from 'moment'

import 'react-datepicker/dist/react-datepicker.css'
import './Scheduler.css'

export default class Scheduler extends Component {
  static defaultProps = {
    onNewConditions: () => {},
    currentBlock: 0
  }

  state = {
    mode: 'time',
    inputBlock: '',
    parseBlock: 0,
    minBlock: this.props.currentBlock,

    startTime: moment(),
    inputTime: moment().add(3, 'hours')
  }

  setModeTime = () => this.setState({ mode: 'time'})
  setModeBlock = () => this.setState({ mode: 'block' })

  isTimeValid () {
    return this.state.inputTime > moment()
  }

  componentDidMount () {
    this.props.onNewConditions({ time: this.state.inputTime.unix() })
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
    this.setState({ inputTime })
    if (this.isTimeValid()) {
      this.props.onNewConditions({ time: inputTime.unix() })
    }
  }

  renderTimeSelector () {
    const { inputTime, startTime } = this.state

    return (
      <Form>
        <Form.Field>
          <DatePicker
            inline
            onChange={ this.handleInputTime }
            selected={ inputTime }
            showTimeSelect
            minDate={ startTime }
          />
          { this.renderTimeHelp() }
        </Form.Field>
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

  handleInputBlock = (ev) => {
    const minBlock = this.props.currentBlock
    const inputBlock = ev.target.value
    const parsedBlock = inputBlock.startsWith('0x')
      ? parseInt(inputBlock.substr(2), 16)
      : numeral(inputBlock).value()
    const validBlock = parsedBlock > minBlock

    this.setState({ inputBlock, validBlock, minBlock, parsedBlock })

    if (validBlock) {
      this.props.onNewConditions({ block: '0x' + parsedBlock.toString(16) })
    }
  }

  renderBlockSelector () {
    const { inputBlock } = this.state
    return (
      <Form>
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
        <p>Your transaction will be propagted to the network { inputTime.calendar() } ({ moment(inputTime).fromNow() })</p>
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
