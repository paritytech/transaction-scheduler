import React from 'react'
import { Message } from 'semantic-ui-react'

export default function Result ({ result}) {
  if (!result) {
    return (<span />)
  }

  if (result.error) {
    const content = result.error.message || result.error.toString()
    return (
      <Message
        error
        header='Error while scheduling transaction'
        content={content}
      />
    )
  }

  if (result.result) {
    return (
      <Message
        success
        header='Transaction Scheduled'
      >
        Your transaction has been successfuly scheduled.
        Id: <code>{ result.result }</code>
      </Message>
    )
  }

  return (<span>{ result.toString() }</span>)
}
