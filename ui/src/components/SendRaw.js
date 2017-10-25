import React, { Component } from 'react'
import { Container, Header, Form, Label, Grid, Button } from 'semantic-ui-react'

import Scheduler, { Summary } from './Scheduler'
import Result from './Result'

import { schedule } from '../util'

export default class SendRaw extends Component {
  static defaultProps = {
    rlp: '0x<signed-transaction-rlp>'
  }

  state = {
    isValid: false,
    value: this.props.rlp,
    sending: false,
    sendResult: null
  }

  handleScheduleTransaction = () => {
    const { condition, rlp } = this.props
    this.setState({ sending: true })
    schedule(condition, rlp)
      .then(sendResult => {
          this.setState({
            sending: false,
            sendResult
          })
      })
      .catch(error => {
        this.setState({
          sending: false,
          sendResult: {
            error
          }
        })
      })
  }

  handleRlpChange = ev => {
    const { value } = ev.target
    const isValid = this.isValid(value)

    this.setState({ value, isValid })
    if (isValid) {
      this.props.onRlpChange(value)
    }
  }

  isValid(value) {
    if (!value) {
      return false
    }

    if (value.startsWith('0x')) {
      return this.isValid(value.substr(2))
    }

    return !value.split('').find(x => window.isNaN(parseInt(x.toLowerCase(), 16)))
  }

  render () {
    const { condition, onNewCondition } = this.props
    const { value, isValid, sending, sendResult } = this.state

    return (
      <Container text style={{ marginTop: '2rem' }}>
        <Header as='h1'><a name="raw">Send RAW Transaction</a></Header>
        <p>If you already have a pre-signed raw transaction you can paste it here and schedule for execution.</p>

        <Grid doubling columns={2} divided>
          <Grid.Column>
            <Form
              loading={sending}
              success={sendResult && !!sendResult.result}
              error={sendResult && !!sendResult.error}
            >
              <Form.Field error={!isValid}>
                <label>RAW Signed Transaction</label>
                <textarea
                  className='codearea'
                  placeholder='Signed transaction RLP'
                  value={ value }
                  onChange={ this.handleRlpChange }
                />
                {
                  isValid
                    ? null
                    : <Label pointing basic color='red'>This doesn't look like a valid RAW transaction.</Label>
                }
              </Form.Field>
              <Result result={sendResult} />
              <Button
                primary
                disabled={!isValid}
                type='submit'
                style={{ margin: 'auto' }}
                onClick={this.handleScheduleTransaction}
              >
                Schedule Transaction<br />
                <small>(<Summary condition={condition} />)</small>
              </Button>
            </Form>
          </Grid.Column>
          <Grid.Column>
            <Scheduler onNewCondition={ onNewCondition } condition={ condition }/>
          </Grid.Column>
        </Grid>
      </Container>
    )
  }
}
