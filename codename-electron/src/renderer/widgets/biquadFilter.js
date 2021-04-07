import React from 'react'

export default class extends React.Component {
  constructor(props) {
    super(props)
    this.state = { freq: 0, Q: 0 }
  }

  async componentDidMount() {
    const { server } = this.props
    const state = await server.send({
      type: "InstrumentGetState",
      data: "biquad_filter",
    })
    this.setState({ ...state })
  }

  render() {
    return null
  }
}