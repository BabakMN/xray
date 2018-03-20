const React = require("react");
const { styled } = require("styletron-react");
const $ = React.createElement;

const Root = styled("div", {
  backgroundColor: "blue",
  width: 500 + "px",
  height: 300 + "px",
  padding: "10px"
});

const QueryInput = styled("input", {
  width: "100%",
  boxSizing: "border-box"
});

const ResultList = styled("ul", {
  width: "100%"
});

function Result(props) {
  return $("li", {}, props.path)
}

module.exports = class FileFinder extends React.Component {
  constructor() {
    super();
    this.didChangeQuery = this.didChangeQuery.bind(this);
  }

  render() {
    return $(
      Root,
      null,
      $(QueryInput, {
        $ref: inputNode => (this.queryInput = inputNode),
        key: 'input',
        // value: this.props.query,
        onChange: this.didChangeQuery
      }),
      $(ResultList, {}, this.props.items.map(item => $(Result, item)))
    );
  }

  componentDidMount() {
    this.queryInput.focus();
  }

  didChangeQuery(event) {
    this.props.dispatch({
      type: "UpdateQuery",
      query: event.target.value
    });
  }
};
