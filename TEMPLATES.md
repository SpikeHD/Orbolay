# Templates

> [!WARNING]
> Templates are experimental! They may not support fancy animations or filters... yet.

> [!NOTE]
> You can find the default template (to use as an example or base) in `templates/default` in this repository!

Orbolay contains a HTML/CSS renderer used for rendering certain components. This makes it easy to change and modify
the look and feel however you'd like! The template can be changed at any time in the settings menu.

If the template selected is invalid, Orbolay will log and the default will be used.

## Ground Rules

* No JavaScript
* Layout and styling is supported based on what [blitz](https://github.com/dioxuslabs/blitz) supports
* Templates require a root ID
  * Exactly ONE element in a user template MUST have `id="user"`
  * Exactly ONE element in a notification template MUST have `id="notification"`
* Stylesheet `<link>` tags and `<img>` sources must be network URLs (`http`/`https`)
  * There is no support for relative paths

## Root Element Attributes

In addition to the state classes, the root element of a user template is given a `data-user-id` attribute containing the user's Discord ID. This can be used in CSS selectors to target a specific user (e.g. `#user[data-user-id="123456789"]`).

## Substitution Keys

In order for your template to display the user name, avatar, etc., you will use keywords that will be replaced at runtime.

### Users

* `{{avatar}}` - the avatar link, example usage: `<img src="{{avatar}}" />`
* `{{name}}` - this is just text, example usage: `<span>{{name}}</span>`
* `{{muted-icon}}` - the muted SVG icon, replaced with an empty string when the user is not muted
* `{{deafened-icon}}` - the deafened SVG icon, replaced with an empty string when the user is not deafened
* `{{streaming-icon}}` - the streaming SVG icon, replaced with an empty string when the user is not streaming
* `{{camera-icon}}` - the camera SVG icon, replaced with an empty string when the user does not have their camera on

> [!NOTE]
> Each status icon is injected as a `<span class="status-icon <state>">` wrapping an inline `<svg>`. Place the token wherever
you want the icon to appear. The icon is only present when the corresponding state class is active.

### Notifications

* TODO

## Available CSS Variables

* `--gray`
* `--darkish-gray`
* `--light-gray`
* `--superlight-gray`
* `--muted-gray`
* `--text`
* `--border-radius`

## Available State Classes

State classes are what allow your template to know how to render the current state. For example, to know if a user is speaking or not.
These state classes will be applied to the element containing the required `id` attribute.

### Users

* `speaking` - the user is speaking
* `muted` - the user is muted
* `deafened` - the user is deafened
* `streaming` - the user is streaming
* `camera` - the user has their camera on
* `no-avatar` - there was an issue fetching the avatar or the user otherwise does not have one
* `self` - the user is the current user
* `right` - the user list is being rendered on the right-hand side of the screen

### Notifications

* TODO
