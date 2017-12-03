# Reifenfeurd
### Tirefire-d (de)

Hi! So you want a real nice CLI Twitter client? This is a real nice CLI Twitter client!  

Reifenfeurd supports:
* Reading, authoring, *and* retweeting
* Reading *and* authoring quote-tweets
* Threading tweets
* Multiple accounts
* Displaying tweets without `...` truncation
* Ensuring URLs are not torn when tweeted
* Client-side thread tracking
* Following, unfollowing, blocking, unblocking, muting, *and* unmuting

And the list is only going to get better!

### TODO: screenshot here

## How to get it
Currently, have `cargo` installed, `git clone https://github.com/iximeow/reifenfeuerd.git`, `cargo build`, and run `./target/debug/reifenfeuerd`.

Soon(tm) there will be OS-specific pre-built binaries, but that day has not yet come.

## Usage

Once you get the client, however that ends up being, you'll want to connect a Twitter account. Enter `auth`, follow the URL displayed, log in and authorize Reifenfeuerd to access Twitter on your behalf, then enter `pin <pin_from_twitter_dot_com>`.  

After that, you should see `Stream connected for profile "your_twitter_handle"`, and you're good to go!  

I recommend checking out `help` to see what all an be done, but you can read below just the same.  

### Tweeting
Tweeting happens in a handful of ways:
* You want to send a tweet out to the void
* You want to reply to a tweet
* You want to quote-tweet some other tweet

Reifenfeuerd supports all of these with the `t`, `rep`, and `qt` commands, respectively. Each command can be used in two forms:
* `<command> and all the words you want to say`
* just `<command>`

The first form immediately sends that tweet:
  * `t just setting up my twttr` will tweet `just setting up my twttr`
  * `rep :1 hey nice twttr` will reply to the tweet `:1` with `hey nice twttr`

The second form switches to "compose" mode, which gives some context for what you're trying to write:
  * `t` is mostly useful to compose multi-line tweets and check your character count
  * `rep <tweet_id>` shows you the tweet you're replying to, as well as the same multi-line and character count support
  * `qt <tweet_id>` shows you the tweet you're quoting, with multi-line support and character count info just the same
  * `thread <tweet_id>` is a special case of replying that mirrors threaded tweeting as the twitter website shows
    - really this is just replying to yourself without including the leading @you

And of course, if you don't want a tweet around, you can `del <tweet_id>` to delete it.

### Tweet IDs
Speaking of tweet ids, Reifenfeuerd has a handful of ways to refer to the same tweet. This is primarily because tweets are numbered from 0, with 0 being the first tweet Reifenfeuerd ever saw. That number can grow to a fairly large size, and refering to tweets by `123456789` can be cumbersome, so Reifenfeuerd supports these ID styles:
* `1234` refers to the `1234`th tweet Reifenfeuerd saw on your current calendar day.
* `:1234`, with the colon, refers to the `1234`th tweet Reifenfeuerd ever saw.
* `YYYYMMDD:1234`, like `20171101:1234`, refers to the `1234`th tweet Reifenfeuerd saw on November 1st, 2017.
* `twitter:9287149823714` is the tweet ID's by that number, as Twitter refers to it

This, taken with tweeting as mentioned above, means the following is a meaningful command:
`rep twitter:20 nice twttr @jack`, which would reply to @jack's famed `just xssetting up my twttr` from the Old Days.

### Tweet interaction
You can `fav` and `rt` (and `unfav`!) tweets by refering to their IDs. `fav 1` would fav the first tweet seen today, `rt` and `unfav` would retweet or unfavorite. The wrinkle here is that there is not currently an `unrt` - to "unretweet" something, you'll have to delete your retweet of it:

Say you retweet this tweet:
# TODO: PICTURE
but decide you want to retract that. Notice the ID line includes `rt id 124` - to unretweet, you would want to `del 124`, which deletes your retweet.

You can also `view <id>` and `view_tr <id>`. These show you either the tweet in question, or the entire conversation (as available) leading up to and including the tweet in question. This is useful if you catch the tail end of an interesting conversation and want to read up.

### Threads
Reifenfeuerd supports client-side thread tracking, in a currently limited form. The tweet associated with a thread is the *last* tweet in the thread. If **you** reply to it, the name is adjusted to refer to your more recent thread. If someone else replies to it, nothing changes. There is no support for following from the first to last tweet in a thread, yet.

`remember`, `forget`, and `ls_threads`

### Profiles
As you may have already seen, you can `auth` and `pin <PIN>` to connect a new account. Currently, all accounts connect at startup, and you see the sum of all tweets from all accounts.

If you want to tweet or interact from a specific account, switch to it with `profile`, like `profile your_alt`. The handle you'll tweet under is shown at the prompt.

To list all profiles known and connected, do `profiles`

### DMs
DM support is very minimal currently. You can only see DMs, not yet send them or look at DM conversations. As with any third-party twitter client, group DM are not supported.

### Media
The best Reifenfeuerd can offer regarding media tweets is to present the link in a hopefully clickable fashion, and permit sending media tweets. As of writing, it does neither of these yet, but hopefully will soon.

### Muting/blocking
Reifenfeuerd will eventually be able to add muted words and blocks to your account through the Twitter API, but as of writing muting is handled entirely client-side. The upside of this is that it can be more aggressive (and hopefully correct) than Twitter's muting, but the downside is that a mute in Reifenfeuerd does *not* translate to anywhere else.
