import React from 'react';
import ReactDOM from 'react-dom';
import 'bootstrap';
import "./custom.scss";
import './style.css';

import Welcome from './welcome'
ReactDOM.render(<Welcome/>, document.getElementById('app'));

console.info("Loaded index.js")
