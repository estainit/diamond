use std::collections::HashMap;
use crate::lib::machine::machine_profile::{EmailSettings, MachineProfile, MPSetting};
use crate::lib::transactions::basic_transactions::signature_structure_handler::individual_signature::IndividualSignature;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_set::UnlockSet;

// User <user@imagine.com>
pub const USER_PUBLIC_EMAIL: &str = "user@imagine.com";
pub const USER_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----\r\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDLHgVeB43L9E1t\r\noJnlZSY6sUFyrInYlm6LHEwhsOjHrpBQQnwj8PobpDV/Ybp4bScBtyqGPm0BNqEd\r\nq5bOlK0eqbR65EFQgZPK6A0J9s9pzxvFNFwuxhUPR1ATdDEw1DfKtV7JWz+MOYjP\r\niKFDZ4A5ojwcfdTnmbui9GaG987tFsbbjYx9ESlRTYkvtjQclN6kzdA04GHnB8t4\r\nUlgRIpLVCshuZRxy8rI6h0X99vag0c86hj3oA18o1SA9W2aoAP6wLMLxs6LBabwc\r\n3BdQFSsgTjxK8EdAhf1xeWTf0ep/hXnk7brHgmD7diC0+twyF26Aj4Gy/QvufZny\r\ndL34gZ8NAgMBAAECggEAL00YduNPdDW6alNCQ5egpX5t/WSM3XF64M6ANEBclVPj\r\nq60v97bAp/s/siByKmVQ9idPsd+LxwKP1rcE8arR/hgLPas2QqdKYbBUiQN/Hebr\r\nUqt05Deg4+P5k+41Hmftbjl6j22+iMtFPv9Ufrv1snZDhWcQU7cLaVF9JuVCvRds\r\nV5b4S8Tk1iR0K2+FyuyRKEyrjtHB3jhVBZvdpNUEoHJotZHdALxFOu/TfG6SGQzn\r\nNP132/9aZV9akWBZ717DxP2pB2k8XPwNfFMKTZtNCz3w6ldvNtlTcWD1O5CkHc8R\r\nACNdb13nHwU/uXfSIq24fzsaoy4nD/Mcz5OPAsf2AQKBgQDVAIjFrCI3GNnEzgMJ\r\nL2ehyw9lxAlMdYBewK0+hgDpuAmaIG/sA+cJpQiZJx2Eu4tySOAylCgDWILmQBWe\r\nTuSkUskMK+ZW2rvaHwhj48Nrmhn6z4RCt92qm3ZAItBC0i13nShJMXAx19fB7Db+\r\n8Cw5QtghDOHhki3jNnA0PiCcTQKBgQD0HqllP3xN/R5q2HZ1ushTadUpzbX2Ao0K\r\nMJF8sqOh162pfacMOeKY3REz9rfsFNyeD6mE2FlBXcLYls2xzxXKyUD9+cvB0QAq\r\nphd9VXru83wlKdQwk6bo4a4XNb7eFVq2cW5UDWBojockmd+dV2Q6KBW/nd+uka6s\r\ndRemcFttwQKBgDyYTlCNy54I/8qxIMP4LG8mqVa2Ej8iHkbWYXKsBI54wKKMH8rw\r\nwUVJIc0QB6G/CMiWWtGIvGlXQMXn7T6ACyOEOZWw13JV/6LpuSVRokJ2MHXdmy6v\r\nx+vFFjrgrIaV7EFfABryaYyEbujIHk0gXjRcA8hDNe9J+qvszLbQBc7xAoGAOaK3\r\nmcj4Xy1grhc3OKqFu3PkOP9xc4i8peg7oTZH/eD/BmI9O1y7TB39fshEOj/eqo7G\r\nFjBCOnWZmCtamx1qZrtHVe9RFQx0Pp2CNDwnTx07dUa/60wg/yCxSpeM3cAq76Iu\r\nSzfxSB5Gd/TAX9SPPE/Ueq4abovEssDeeZRTccECgYEAg5wxn2PDek4gomtucnRv\r\niyQ1oeehcYkaqtH4xV2PSrkkUqPUHb0Wj2FXVDl72BEUFGIucDYp9uw04lDui9qj\r\n5cC2eJImcKhQB9x1MGWKUBrZjCX1uX1Y/quD7XD80ojBAnigo43vJDCz0RSHXZQf\r\nWf/0/HcnhZYFpj2OU+/AF6c=\r\n-----END PRIVATE KEY-----\r\n";
pub const USER_PUPLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----\r\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAyx4FXgeNy/RNbaCZ5WUm\r\nOrFBcqyJ2JZuixxMIbDox66QUEJ8I/D6G6Q1f2G6eG0nAbcqhj5tATahHauWzpSt\r\nHqm0euRBUIGTyugNCfbPac8bxTRcLsYVD0dQE3QxMNQ3yrVeyVs/jDmIz4ihQ2eA\r\nOaI8HH3U55m7ovRmhvfO7RbG242MfREpUU2JL7Y0HJTepM3QNOBh5wfLeFJYESKS\r\n1QrIbmUccvKyOodF/fb2oNHPOoY96ANfKNUgPVtmqAD+sCzC8bOiwWm8HNwXUBUr\r\nIE48SvBHQIX9cXlk39Hqf4V55O26x4Jg+3YgtPrcMhdugI+Bsv0L7n2Z8nS9+IGf\r\nDQIDAQAB\r\n-----END PUBLIC KEY-----\r\n";

// hu
pub const HU_PUBLIC_EMAIL: &str = "hu@imagine.com";
pub const HU_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----\r\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDOoGyshmjFsQQR\r\niruJnNHkEbRc/HbRC9IrFGKiaMYOfPha5VLW/G/6zfR9mRNJmw2YCufuoDiXwO93\r\niC0dS+TtBws8ug+dvo+g2U1wKBCXhnOmq3wek+9kYWuDuKZetYyKc8D49ycNCFAd\r\ngBNzBgJOqIHM0sotLFbeypn1hPJ6eBvtZopSzmz+sIoqOnsphA9uvxBNv+IU3q4t\r\n8i35dpp84J/sEPmQOctCLdbMXkQUi4tOA9tOylhtQzab/zu3xVZyAKsNfIj1b8Yr\r\nD4pghD5dHcFpdmLy7xKz8SBpo8kvu5WGX3lqmg46dy5Epiy7+XZZ8kHXI+GDco8q\r\n3xXjAmrnAgMBAAECggEAbJO0R3xhtfBs2OLz5T44tQXwpyQrM3JowCZ4Jiu0V4yn\r\n3OvAeiQwm7R6Ez3K+tS1cf3ZHnWnB2dk7oTqYAivhTY8W27SIewvATDvFk6XWWhX\r\ngW9/S3olQ7RFkPQIyJ/R+DcXYjxNkvWZU8cncTvpHnhBrOXfQ5mXFH+WI4P2/w8I\r\ntBRd1mSbmjZ5FjDMCsc4Bu2l2vhuyoRTfJ+kGIMxLAAAuJPjPGCM6rA9ErNvbF4M\r\nH+73m8Gxtk2oa8OzB45IW+aGe3q0bvIHAeTYsZMt753z3qttqfPo0GLR34enL+2X\r\n13mcXnub9y+ScmkqHPv65QXYz88WeH1xyEZTV8ALYQKBgQDjtZIFi5aadKDw2SeU\r\nh2TaBkP2mLSHMOl+IZ9R85isqu8GX/YPDyfjb5boqyrLDwbC2B9jmclGzRmgDxv5\r\nifW0ul1Z7MDltKt17VLt2ztpiM9azkLPU75N/4SqiFAPiKhLh3rs/KowVneWaejV\r\nwkZ1cRQsEAo4WEE0Yl0fLem7HwKBgQDoTE8weZezn8RBsuL7XIiYz0410RErgFas\r\nKwvGCHWIfZfEJxPLxYF1y80ux9R4Q8lvwyof0jGhH/OZe8ZKhmkQFz4zmfO7kEnh\r\nsEK5lUSNn7iw4O8vQd8kJpAJNma4ZFwnxzBnFQQNn6Ra9UfW7+anSDiJ992f5Qje\r\nczphaNEfOQKBgC5f1RxCAGr2Y6yJXTE3ncd1TTQUh3ec84CXkl3bUXWg3ksbEf6h\r\nJIuCN2atLWrrZYIbB9F+CWrc7GIXkafxe9PRvJ9Kw0JE2EKNEb2VT3U/wpMIvLyC\r\nnpg9+KPOXRe8yiGPtu46yuJLSdGdQij+huD2JZiPr7Un7Ceh/LsT260bAoGAC4M8\r\nqXpdOlnAsEDdXvfHDUu45JHn9+/0W7QGcVoZ+RnAW8hUAtVXBS+Ei7z7mrpBUXiq\r\nrckNDJ84w3KO8UKYEmQUgRowKiuMfdLue6QaMaqozJtZP05UcMY4fTxk+t5+cro8\r\ne99exA4VZyyg0tYw6Dl8E9pk8Xe4aM3tJsa7FoECgYEAxdhdmO3Ug32gJ453fwNq\r\nPGv4yToQ9iXhVHzkfA2sF1f0ncYjTxr44iabJN2ONZsd717mB868D3XHW4DpXb9b\r\nFgjcWf/Wxe9A0D8JErOlBKy+8WjxNfHRddNen7L8VSGAwspDeLREch7dkHBlPZLN\r\nD/wf8d/pmsfFO6lQDHNOAUY=\r\n-----END PRIVATE KEY-----\r\n";
pub const HU_PUPLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----\r\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAzqBsrIZoxbEEEYq7iZzR\r\n5BG0XPx20QvSKxRiomjGDnz4WuVS1vxv+s30fZkTSZsNmArn7qA4l8Dvd4gtHUvk\r\n7QcLPLoPnb6PoNlNcCgQl4Zzpqt8HpPvZGFrg7imXrWMinPA+PcnDQhQHYATcwYC\r\nTqiBzNLKLSxW3sqZ9YTyengb7WaKUs5s/rCKKjp7KYQPbr8QTb/iFN6uLfIt+Xaa\r\nfOCf7BD5kDnLQi3WzF5EFIuLTgPbTspYbUM2m/87t8VWcgCrDXyI9W/GKw+KYIQ+\r\nXR3BaXZi8u8Ss/EgaaPJL7uVhl95apoOOncuRKYsu/l2WfJB1yPhg3KPKt8V4wJq\r\n5wIDAQAB\r\n-----END PUBLIC KEY-----\r\n";

// alice@imagine.com
pub const ALICE_PUBLIC_EMAIL: &str = "alice@imagine.com";
pub const ALICE_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----\r\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDRc91fjjPKaFK5\r\ntCpYU0DP+fAkGOG1WV7UT0G+ShEQZjNxsHzZBZEfIEY3r8e1l2Y3mBWTGm7OZjwt\r\nMheI/otuKjy7z3kvQFNwIqu0naav0H4Ni+qQVw8MESiQCdl+rSTG8pfnytv4c5va\r\nosBTPipjttG05j1Dyu3dvhCyAVyf28uMgXMgi3ZHiEgzkBJ9gRk/Y0XSLw5ej4F/\r\nYAOAww1xtBIev6yMN7T6SIs4J1u4V8lYjS29b7QtnHhv5DlOv5jbbrNvUzBxIMux\r\niVpPd7sIsb6FcRv6ktXkMvX3YrtcvIwCSjtGuNkkR8Atjg17TvWIVFEjiJGiSbT0\r\nporyVALJAgMBAAECggEBALW0la0rOYT0IihWw5GikAydYRZ/u3fPU3ROWdOSf+FB\r\nOIn9uSGxMafWIPR4e4HvGU9LkhPvD4fhq3zuAvrnEOD+pXetUw2J+ZjDi0CCsDPA\r\nZUqtQk38KOKoFA95bAT4gnpRxTTiiRiuetwe60iMGswssxsDzsx8RQVkuSkkA03u\r\nFym4mJv0HiRtJvnduMYuPvz6uROQUBX9U6A3PzDofpePSIDoUjvBdC3bkrO/k3xY\r\nub6I+AdhEpKXxS6phSqmWSGVljNTd4L932TVHp0Z6PXP7oA8cekAAOSTMitMWebB\r\nHa9GBvGmq07sT/l74bYY7ek6xQzO59p/RZ25SKge6bECgYEA8hBZ6HhvPvtkmSKm\r\nnMtPn/dGr/qZx7NG5yEGcR/ZGl82Z3BKsGIFZd3LJPgX9qHFVW/scUr7gTZrwuJn\r\nALr+ZZ4i6FJuDmtkK0xDFUPnhwLZ+3SbecxKpbAs2Fn6Xck5UCj6KMc7yK9+RQAr\r\n/xTv0fDuFXKcXD6apDe2DH4CtRUCgYEA3YLfoLekJaT7ATsd8Kx72AM59w7pCoC8\r\nZCY5erY2zttentWEvmfbj4Z3UXpYmlnKqxjO0GR0YSDrafbW+UcW9rb1tkwgC/Yy\r\nQwr2PdS0awLFwRxgCCRbxDVR/K4WeEf5vg5BsbgDL5SgkNTjUYDikoFYvetpqMhF\r\n8NUBuDHnq+UCgYBIHN7sBpT9ql75z/za0SbTfRMt8fZGK/5/dLM6mEZROPQ4NJfL\r\nnCgHhN+0D8Tz0JW9Mi66QaBTiiboVBJjgVGwbD/x/jyTRyL8UyfY+fXLnunZo499\r\n5YKHgciaW5PexMeyPcLoyxHgY4e2fqqcv1wCq0gCU7aJNI1VRtORPSkJUQKBgAiC\r\nvMDaDW581kAH23Chz+hOx21cn0uOAq7+YPr8AUMVXp9PqNf/+YmcKv1wa/MSPB0y\r\nyM9s7KPGtgTOPRg6yQEVqn8kkZ6kCO5Wf3uErqHl618uDeqCKxxwyAOjVK2uZkQN\r\nHC0N2uF7HmgyJcG95/alDZnOb7LSbw1/wZ8oG7SZAoGBAPAceOKPvHd3eF78ziI+\r\nmDP1B25HlH7Q9H6e71wAmBg8iVlVbSlj6/Y36neL1JqyV++AK7pVCWJ+HghpiC+7\r\nnnpN79VGfs2K7GBeJy8jekPrk8YRjOzuU4p3GgLf2i8M9VFcxUIpIkem3nAzWT2d\r\n3qlrVD64dMLi0rn7OWDHmVWw\r\n-----END PRIVATE KEY-----\r\n";
pub const ALICE_PUPLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----\r\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0XPdX44zymhSubQqWFNA\r\nz/nwJBjhtVle1E9BvkoREGYzcbB82QWRHyBGN6/HtZdmN5gVkxpuzmY8LTIXiP6L\r\nbio8u895L0BTcCKrtJ2mr9B+DYvqkFcPDBEokAnZfq0kxvKX58rb+HOb2qLAUz4q\r\nY7bRtOY9Q8rt3b4QsgFcn9vLjIFzIIt2R4hIM5ASfYEZP2NF0i8OXo+Bf2ADgMMN\r\ncbQSHr+sjDe0+kiLOCdbuFfJWI0tvW+0LZx4b+Q5Tr+Y226zb1MwcSDLsYlaT3e7\r\nCLG+hXEb+pLV5DL192K7XLyMAko7RrjZJEfALY4Ne071iFRRI4iRokm09KaK8lQC\r\nyQIDAQAB\r\n-----END PUBLIC KEY-----\r\n";

// bob@imagine.com
pub const BOB_PUBLIC_EMAIL: &str = "bob@imagine.com";
pub const BOB_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----\r\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDL+Q36e5OGXs+U\r\npLIvvWqf2+HvpfxyUpD7K3fPirp1mb/8QmwI49B9oUJi8rAzbO8iwZ2+TRRMejXP\r\n92rRouRZiICul4iJhgZP+C3a8Did/NO1Rd1ga2frlKFdW9FLRz3sSueBs4XHAEz8\r\nMO1bZKeAgG76wDE8sPHWPD1Eese3xm/twQh75rv533/635GlTWbVA+t88Y/3nBNU\r\nibcpcBFNXER3zgQHen/ziYy0onGpfroSBvEAMnFWPFxMD8jiyHTGQJxfKfpo6IL0\r\nJpUsBqTAOHVR/zpOBNyNfBCjKljSt7hOT6RMmkKNBkrZW7CqOoT/DP6fjXCii5qo\r\n2+1R1I8rAgMBAAECggEAQHDZ0rXyALS5fiueQ+ji48hTFCRroi6lsUSlYINirz64\r\n4diWSyS2PMqSr7IeqtCSqHdk/7dYX2UI1UBEliCRZhvzE7W6JECcg+1Th2T3bipv\r\nNEjzTMACH/JgHJ+ietbyIFH13As8i4dzywUmKAafyMBKz5uisprwfI+hh54GtO9C\r\nz+YKBEnQK8NPH7t4aAWkLdu7syrZhfdR39hY8/x7i8l2Crmk0661eYok3uX/0wzX\r\n6A4Oor9wGtlfZmaRoFBV1yZ0waY09FwAR8NnwEUy+RGX4ibFFlfhdqILvpNTUyBc\r\nfNnsXq3sgbuAwFtu3E+bq7cr2usJP9tF9kp4YPB5KQKBgQDXcDBfp7CV0uDY3jl1\r\n3rijEY92S6yf2I5hdXGmKKonH8W2BixFbNKxLtiW30KP576CH48XSoaqdC3Jn7kd\r\ni1kv4fGCSvHt1oa8dvS9Meuh2kzX2t4NJcl3hH4bMeUhLcUd8UR/oN33WAHlcePf\r\nTcMpso+7kaKWwIkhk+07HkbsnQKBgQDyYEBhSS25LglP5QPgaMYLt3zL8/ksmMGC\r\nQUmWXwHjAOdoZaR4XyPhT75L0Dhk5yTprukr+7Nh9pNWFEjSNBxXO/edPJnK75I4\r\n++5nNRugY7f+zJhQhGtrYkrFy7BK0MhSgz5j1PdkZauHZR3XUupRoEIqolA8NDmR\r\n5uuFyMQMZwKBgBShhcBjSqHOZAHgphgHkB7tm5N0g796+YeTu6Jx2nmMrV5VEQBE\r\n/5hAKDWqg7FMPA4x/333gRXpskjQpuWRZqUTOhGEI87m8Fgz/BPyJ+KECT/Skdko\r\neTac1Ya9LHgU7f+ED085lIgPQX87fNrxk3L2ypTnyW4uWfOBOwKiT6BxAoGBAMDQ\r\noWbDeIRggzfz0Gmt8B9SEQ2PQSKhQxAEMC5X1oBoL691bKn1xe1wKsrVEofy+gKN\r\nCyHaUpIUPpG0AVp36jPbmNiVZSN0ArcidD3Wmeu2aKFi0aj8Lxh2UVWY/N4HydUa\r\nY3+35DcSSqqjXmH1rELTYs/X4EyEn2fadHMxjATxAoGAEoOq5yb2qjOP0iq/YORn\r\nLyJg/kiRW50hv6TCLUT1N+C3df74+1YUK+oLRjIGjTIj+vtiqzS4u/xUr5+RIxyG\r\naa1Pf2IDLeAB/sJ5jyaiaTUSBUdKOvS6rJ5XdNFMfDQu/6lYtZQqI6o16G0aa1ca\r\nhSa/0xYbsniBPAdQqfsTgqw=\r\n-----END PRIVATE KEY-----\r\n";
pub const BOB_PUPLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----\r\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAy/kN+nuThl7PlKSyL71q\r\nn9vh76X8clKQ+yt3z4q6dZm//EJsCOPQfaFCYvKwM2zvIsGdvk0UTHo1z/dq0aLk\r\nWYiArpeIiYYGT/gt2vA4nfzTtUXdYGtn65ShXVvRS0c97ErngbOFxwBM/DDtW2Sn\r\ngIBu+sAxPLDx1jw9RHrHt8Zv7cEIe+a7+d9/+t+RpU1m1QPrfPGP95wTVIm3KXAR\r\nTVxEd84EB3p/84mMtKJxqX66EgbxADJxVjxcTA/I4sh0xkCcXyn6aOiC9CaVLAak\r\nwDh1Uf86TgTcjXwQoypY0re4Tk+kTJpCjQZK2VuwqjqE/wz+n41woouaqNvtUdSP\r\nKwIDAQAB\r\n-----END PUBLIC KEY-----\r\n";

// eve@imagine.com
pub const EVE_PUBLIC_EMAIL: &str = "eve@imagine.com";
pub const EVE_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----\r\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC0xe8v3hR6mFiP\r\noqLB6f0zQ5g8AwQXsQt8IY7Ew5qwmISPYrRVBCXnwT30p/4E9PbtM4w2vMTxeNEr\r\nCZZUBD2g7smDws9aaMBK9bDMN7NPSZWHY6pklO1ZoiTAB0rLHcNMCKWxh6MGOD0c\r\nhyGrZ8e51gFaKeY8g/Lf52J4/ikAHe0qtURxTQymF8gfHAXqgv1FRQKefkEiBSZR\r\nm/FSN7kUEq9wc1ihZYhHxocD0rKKBI4B5ukwpSm/YphVfGQpH/0UEAiH/Mbah63E\r\nYNZHLcz/E0SXPx8yY1uiNwBHsTxdvdjOZfjRA6NI1zt8jzU2/qnLviMJGhR8kmUC\r\nM1KFXhbXAgMBAAECggEBAJwa+UwqySB9Aq020roF6AMm8gWQzS3wU2ykRfeguqqZ\r\nao+vuu5XoKwbcfceQvvg1oNLql9yb9febzJtGwX+i4G38wj3Z7w5DSUuObiAuTVU\r\n4+2AoZouCTEIFhhFs87nKsk0BnHiIWOzmQJTpz2vZwgRyCUmXY4Qm+HCnITDZoCT\r\n4ggp2qSSmkbNebnpVoAWiVrq9PLJaGSE2VSeNAhN23HAkCFJfxyIZ0K2T1HhUcET\r\nx8pzzjQ8+5olXdAhpHlhn1R6KrPd1gDDdFAHUlCDDVcJwZ23IrwctevJCbJN4w22\r\nOnc2XG3zo5I3/vviByw4wUhlQt2rs1UpAaszKC0ZlkECgYEA7Ky7nJfeCjF3ubDO\r\n/r6Q55te00IFp4K7G4TSC/3VshzlkW/naiL79idfMM4p3I3RAbln/He4jkQg/mbS\r\nMRSvWUpL7AUhRncrh2fW/KkVhNMZrW+ZQXKKaTRBlqS17AZ4qfb70mYKx9OHCOd7\r\npvgtLtdL3vwsC71BlMBDxp382bECgYEAw4itnzUni1jJ/tGV9ANPtFSheXTZ0l1b\r\n39IQqq9zEEtuEL103ojx9nXtCBkM4xxEAmhOd3AujQdJ7LRgzvG1YknF2iJJ1on7\r\nzfnEIO0Keh6JDZu3V8AMipGk8V7FHeWwkZ41F3ZA1iX7ph7rWwL90kUyTG7Jm1kv\r\nCaZvF6sMEwcCgYAdsElUimRrCOuI2poMsKECvKW3gHevHKIWAKJMqMnOrvtJNC1m\r\nTf8nUKcLO0FbDlsiiTx3GhmHlxpAb2t8hqi8XaqKb9ZNvZMzSB5a0WcGo2h8Lhye\r\nbhzYt5pmqEC92831HCtYqD4/9NPilQ8Y0dbxIka9MQrhahs46qlV1+mo8QKBgFS5\r\na2m50XmqrlPQoqYJrUaZCSKkhCGHvGB+GCStQzFDTdzJCtPGCPne3ScOG6xouftF\r\nEQLlw/Xtu5VqDyx8RTjieG1tQAtK3KXCXL5bl5eUlZQk7cuC9qwwMYU4qDatKNXf\r\n9GdHIQ7phGHVsetMn4i13PaLZqX+fcgzqp1ZyzCPAoGAPFbTIHFuulJ9OpIzPmdm\r\nKSX6EVKwTabV2rrpHf2kazZgxFvY1X1oqxt6SekBjWY9ap/RbXkLVGESKzkzwlGF\r\nK/CBEup0zy8ZVjPPq5+zojnGOTc00hoAvie+UiBORO+wpYalIA7nT+HXjKMHZbS1\r\nHIxHTahNFadOtOALkf9MgYo=\r\n-----END PRIVATE KEY-----\r\n";
pub const EVE_PUPLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----\r\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAtMXvL94UephYj6Kiwen9\r\nM0OYPAMEF7ELfCGOxMOasJiEj2K0VQQl58E99Kf+BPT27TOMNrzE8XjRKwmWVAQ9\r\noO7Jg8LPWmjASvWwzDezT0mVh2OqZJTtWaIkwAdKyx3DTAilsYejBjg9HIchq2fH\r\nudYBWinmPIPy3+dieP4pAB3tKrVEcU0MphfIHxwF6oL9RUUCnn5BIgUmUZvxUje5\r\nFBKvcHNYoWWIR8aHA9KyigSOAebpMKUpv2KYVXxkKR/9FBAIh/zG2oetxGDWRy3M\r\n/xNElz8fMmNbojcAR7E8Xb3YzmX40QOjSNc7fI81Nv6py74jCRoUfJJlAjNShV4W\r\n1wIDAQAB\r\n-----END PUBLIC KEY-----\r\n";

#[allow(dead_code, unused)]
pub const PRIVATE_KEY_2048: &str = "-----BEGIN PRIVATE KEY-----\r\nMIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQC4TGikJzuhYXpL\r\nxSe4auBjxub7h7k3vCjkVoEOdG7vFwIaxQdDknKL/jyv5v+PUlr3rrkU/S57fBap\r\n4lOuMAOnUJXQcFFTCcrIjr/iFv/SLvHp7s5AdG6dKoobFacF4enwNLYdY1Nsb6Wy\r\ncVjK/E/uhW+h/+gWdjL3imfLPOzP1vYe5HG6Gkzu+ddd1LnQJsRxoUSODkfcMvSr\r\n9OPcqCkn2t7LSOxF9yGvBPcs7toN3qe5pcnIhkkxxmmAhPngbp5lbWGjtJYmY2Ow\r\n5oD77hc5jhsi98UD4YysBvVoSkXIuXl6w+NbfDKLsBo1IhqxgsCu57LV2aHJ1ls/\r\nRB5jiic5AgMBAAECggEBALg5sPQ+P45PKXeqSc1AELPMdMKEZnI/RUUS74jqfKXF\r\nAxaNU3iJYLVt224eY+H5efNSlbJUb22CmgkRs4JQfqZ2mHs2eySdijY287pmMS0C\r\nPlIQo92sRZIXntv6Je5saHPzzQPNcOvZIvIf+ZlW4/PTMMboTzB80O+/S4fOjA4o\r\n6Tjypw6wvigQyD2GRo66PEHPWMLlNjAxa+7nQp7BXC0UX27y7+cjecDyaCG/pUbX\r\nAdbgNSJSIp6n55zchVsmmrinOkeN9MvcASIgnCROS9al5yrevfLxg3fEwW4H8/3G\r\nfdhYzJ5g386mR6EKDe3gX7/2HoUd5UoTzb4sNhKp9vECgYEA48kf3XOcJC1+vyQo\r\nEWaK7JoGPSyqaGD5WCkRNDsYHukYx//ErNRApHCeV9XrpEKBlp0eRQA0U+z5g0ix\r\nVhx1ZyaN3clFcReXP8C8uTyBMFdQGGWr+QuTg5S/gJjs9aqI8m1SKlpQfTath9bR\r\nscMt4+2TZcBFmRZVqZCLaUlwKHUCgYEAzyBYfLbZVM/5kkVJRqAOXgqqaU+2LG9V\r\n1yRKQR/BYiHmuNwFfTarWENd8mn4SLO5eXTWGHI2R6ANSTy6/CPuoFbg6Q6I/ztJ\r\nrzShuTEWkE7X3/MgJONNhaEospuc4E2Kqzwjo2nicP3P+CXvOHirEYwmPRrIIGHX\r\noFoNfCKpyzUCgYA8wFx+TKI9R+EBC5ygH3A38FBvqmT8l7iI2dMb0hL504NnfACx\r\nc56V/O9OT+CcG5zCVb9H+ej65T4a1J1vcQGi9DZsC404v2j4eOgco1V1ViQnjZ5T\r\nOtIqCtcUbjTsxIHn3l5Gq3XCH34it5mPxpWLr8ZbIe+uB7XrFoEIIK0ILQKBgQDD\r\n9rFhhUnH4WEZj64NQN93ABZMvtr33XpUq4QJa3b2VmbJHXmgBvpD7rDS6om6lzfy\r\n/qSUynIqf/YyBWBPr9tUHf564YKiIEDNoDkmUpgrfjzmKEuQOvIcbOZpXasl2JdK\r\n/QIm2MYh6zE5cQKM5jXLy1JeW5lecdOlZa3+dXk5xQKBgQDgDg5ppiD8DGj2enKA\r\n+onlrvyXmFKT3swMWQouR2VV8r/0Sj9ogvExT6Pwyt16EkmLlrls7GvdGy97iKuZ\r\nAPSD18HuI+9OSjqhRHm5BzNAFGHNRfhCe6locIgaHH601QNLWc/4uwzAZ/s+iPmX\r\nyI2vSaNkQboPCdyJoOECRyNBEA==\r\n-----END PRIVATE KEY-----\r\n";
#[allow(dead_code, unused)]
pub const PUPLIC_KEY_2048: &str = "-----BEGIN PUBLIC KEY-----\r\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAuExopCc7oWF6S8UnuGrg\r\nY8bm+4e5N7wo5FaBDnRu7xcCGsUHQ5Jyi/48r+b/j1Ja9665FP0ue3wWqeJTrjAD\r\np1CV0HBRUwnKyI6/4hb/0i7x6e7OQHRunSqKGxWnBeHp8DS2HWNTbG+lsnFYyvxP\r\n7oVvof/oFnYy94pnyzzsz9b2HuRxuhpM7vnXXdS50CbEcaFEjg5H3DL0q/Tj3Kgp\r\nJ9rey0jsRfchrwT3LO7aDd6nuaXJyIZJMcZpgIT54G6eZW1ho7SWJmNjsOaA++4X\r\nOY4bIvfFA+GMrAb1aEpFyLl5esPjW3wyi7AaNSIasYLAruey1dmhydZbP0QeY4on\r\nOQIDAQAB\r\n-----END PUBLIC KEY-----\r\n";

pub fn get_hu_profile() -> MachineProfile
{
    let profile = MachineProfile {
        m_mp_code: "Default".to_string(),
        m_mp_name: "Default".to_string(),
        m_mp_last_modified: "2022-08-29 13:09:05".to_string(),
        m_mp_settings: MPSetting {
            m_public_email: EmailSettings {
                m_address: "hu@imagine.com".to_string(),
                m_password: "123456".to_string(),
                m_income_imap: "".to_string(),
                m_income_pop3: "".to_string(),
                m_incoming_mail_server: "".to_string(),
                m_outgoing_mail_server: "".to_string(),
                m_outgoing_smtp: "".to_string(),
                m_fetching_interval_by_minute: "".to_string(),
                m_pgp_private_key: "-----BEGIN PRIVATE KEY-----\r\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDLHgVeB43L9E1t\r\noJnlZSY6sUFyrInYlm6LHEwhsOjHrpBQQnwj8PobpDV/Ybp4bScBtyqGPm0BNqEd\r\nq5bOlK0eqbR65EFQgZPK6A0J9s9pzxvFNFwuxhUPR1ATdDEw1DfKtV7JWz+MOYjP\r\niKFDZ4A5ojwcfdTnmbui9GaG987tFsbbjYx9ESlRTYkvtjQclN6kzdA04GHnB8t4\r\nUlgRIpLVCshuZRxy8rI6h0X99vag0c86hj3oA18o1SA9W2aoAP6wLMLxs6LBabwc\r\n3BdQFSsgTjxK8EdAhf1xeWTf0ep/hXnk7brHgmD7diC0+twyF26Aj4Gy/QvufZny\r\ndL34gZ8NAgMBAAECggEAL00YduNPdDW6alNCQ5egpX5t/WSM3XF64M6ANEBclVPj\r\nq60v97bAp/s/siByKmVQ9idPsd+LxwKP1rcE8arR/hgLPas2QqdKYbBUiQN/Hebr\r\nUqt05Deg4+P5k+41Hmftbjl6j22+iMtFPv9Ufrv1snZDhWcQU7cLaVF9JuVCvRds\r\nV5b4S8Tk1iR0K2+FyuyRKEyrjtHB3jhVBZvdpNUEoHJotZHdALxFOu/TfG6SGQzn\r\nNP132/9aZV9akWBZ717DxP2pB2k8XPwNfFMKTZtNCz3w6ldvNtlTcWD1O5CkHc8R\r\nACNdb13nHwU/uXfSIq24fzsaoy4nD/Mcz5OPAsf2AQKBgQDVAIjFrCI3GNnEzgMJ\r\nL2ehyw9lxAlMdYBewK0+hgDpuAmaIG/sA+cJpQiZJx2Eu4tySOAylCgDWILmQBWe\r\nTuSkUskMK+ZW2rvaHwhj48Nrmhn6z4RCt92qm3ZAItBC0i13nShJMXAx19fB7Db+\r\n8Cw5QtghDOHhki3jNnA0PiCcTQKBgQD0HqllP3xN/R5q2HZ1ushTadUpzbX2Ao0K\r\nMJF8sqOh162pfacMOeKY3REz9rfsFNyeD6mE2FlBXcLYls2xzxXKyUD9+cvB0QAq\r\nphd9VXru83wlKdQwk6bo4a4XNb7eFVq2cW5UDWBojockmd+dV2Q6KBW/nd+uka6s\r\ndRemcFttwQKBgDyYTlCNy54I/8qxIMP4LG8mqVa2Ej8iHkbWYXKsBI54wKKMH8rw\r\nwUVJIc0QB6G/CMiWWtGIvGlXQMXn7T6ACyOEOZWw13JV/6LpuSVRokJ2MHXdmy6v\r\nx+vFFjrgrIaV7EFfABryaYyEbujIHk0gXjRcA8hDNe9J+qvszLbQBc7xAoGAOaK3\r\nmcj4Xy1grhc3OKqFu3PkOP9xc4i8peg7oTZH/eD/BmI9O1y7TB39fshEOj/eqo7G\r\nFjBCOnWZmCtamx1qZrtHVe9RFQx0Pp2CNDwnTx07dUa/60wg/yCxSpeM3cAq76Iu\r\nSzfxSB5Gd/TAX9SPPE/Ueq4abovEssDeeZRTccECgYEAg5wxn2PDek4gomtucnRv\r\niyQ1oeehcYkaqtH4xV2PSrkkUqPUHb0Wj2FXVDl72BEUFGIucDYp9uw04lDui9qj\r\n5cC2eJImcKhQB9x1MGWKUBrZjCX1uX1Y/quD7XD80ojBAnigo43vJDCz0RSHXZQf\r\nWf/0/HcnhZYFpj2OU+/AF6c=\r\n-----END PRIVATE KEY-----\r\n".to_string(),
                m_pgp_public_key: "-----BEGIN PUBLIC KEY-----\r\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAyx4FXgeNy/RNbaCZ5WUm\r\nOrFBcqyJ2JZuixxMIbDox66QUEJ8I/D6G6Q1f2G6eG0nAbcqhj5tATahHauWzpSt\r\nHqm0euRBUIGTyugNCfbPac8bxTRcLsYVD0dQE3QxMNQ3yrVeyVs/jDmIz4ihQ2eA\r\nOaI8HH3U55m7ovRmhvfO7RbG242MfREpUU2JL7Y0HJTepM3QNOBh5wfLeFJYESKS\r\n1QrIbmUccvKyOodF/fb2oNHPOoY96ANfKNUgPVtmqAD+sCzC8bOiwWm8HNwXUBUr\r\nIE48SvBHQIX9cXlk39Hqf4V55O26x4Jg+3YgtPrcMhdugI+Bsv0L7n2Z8nS9+IGf\r\nDQIDAQAB\r\n-----END PUBLIC KEY-----\r\n".to_string(),
            },
            m_private_email: EmailSettings {
                m_address: "abc@def.gh".to_string(),
                m_password: "123456".to_string(),
                m_income_imap: "993".to_string(),
                m_income_pop3: "995".to_string(),
                m_incoming_mail_server: "".to_string(),
                m_outgoing_mail_server: "".to_string(),
                m_outgoing_smtp: "465".to_string(),
                m_fetching_interval_by_minute: "5".to_string(),
                m_pgp_private_key: "-----BEGIN PRIVATE KEY-----\r\nMIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQC64vdsZvUZE8j2\r\nHQUyA38r4WvBXvZGCbayV1fMlJCCfLTyAfx/3hjlpgNQ8Gig60wWhtzp0JpvudJC\r\nRLZdmOa95IxQd83Ccw1uqW9x3So7lXhv38Sh+aPGsiNATIV4D3MrbFK3o2CSia7w\r\nAuGo209fTVBrAkmbG9XCXtX60AlwNol+Zy0fqZK7fZka2VMqt3loP8AeKzM+vYWE\r\nKnUAqv8KQZqS4IDY0BDCymAf/5m1WTm3ztwatVZCHlfZRc/J4NxN/kdlecnfjWQi\r\n7FJT1gcOlHnxo4hPaP/lyXwThbMeL6UrzOSOofLUXeWQFpvBgxhxwI6DJTdQSLBs\r\nMRx3206zAgMBAAECggEBAKPsKLIArMNOQ1r8oW34+zb2BpoaPVG5e3J6ghyDwy1x\r\nTRVpAJz3pkboksgP1vYu7RJlQKglvRB6oR5XPs5iKyIssZZzPxtr50BFhecN4tlY\r\nhcc7MzIP0cOaxKjFddyVUKOp4/QHbdGaysLjBCQkGT6yhfMWkpFmnNxcarwQdfbh\r\nnkQhOCaORcfcpQxwK5vQz/KsF16wH+lz60kBcFEySAeI7TDw0ep3UhOr9Tx8memu\r\nPOrbP4/+iXpWfaTGkZfjVhcPxjob8OURDzv2KcqOoGNpHmL/4vtyrviqlsGbKm7F\r\nZSbqFkG7gTy9QtgOQfi+58agV77uTtiaAWbdK8VIyKECgYEA7UeXWOGWhRqyQ13Y\r\ntLXh28M9DeSgX0s7wwE6AyElDbWapfHyvtMUXwdEyT6aAUz3aa2V6pBVR6hyPM/V\r\nfkyhbydRa8M83CWXqpye+tc6r7Ku/bg5CMjObdaHSHLU+LkX2s9eNI0TeGWZU9K/\r\n3ajNWo8la89P/HGzQlzk6Z2pS/UCgYEAyaGSQ4sN4sJKfBYVQ8jy2Vu1CZIPgp/2\r\nKzK9K9X/yFnnJxqtFOUKJKciSsK/ywGULZek79oCvhizmvExg7+Zdy/ypWaNd/Sz\r\nWYW1Qj/p8JU7W1uJbn+4ZcuqqdslbNfrpgmjTchLroA7XtKDrWjZglzp/GyWRMLK\r\nZhfaxHM/bwcCgYEAzGMSl1kaUuVAEI9SD7dsKeTvPnxlODCR9dOkqPVv+XMpFzBm\r\nLMGdlo2oTsFB30TxCXKg5EAXdXY/kOpluDlCBYEUvYKbdfZbwnbO6rtird14psx9\r\nNHfkePCF734avXSSe8SMHTA4SUka3f13j/PLj+omDcux1n4KL2vdMu6/2dECgYAu\r\n6kJPJv7PIWgVYUoHYK1o99ay6GJlgXTU7lRn6749TvXi+mkFcJmgl6b6AECCKtbg\r\nmOVOzcpPkw3PYomj3yQFQInUBH2sSKqmjN71EEwNp5uNEUp0BJHSVcZbCVu27LKv\r\nCpUN1yoM61dlI9Rxt/DMTXRAQL/iNfTENo63oR1EZwKBgQCwUfswJspvZdh/NWgr\r\nx/3sKxm+zk1GxviwC8w6y3EkNkpGRCcvLHxNxXgcei1Z29xVXB7pW/kHD4GWBNyX\r\n+nPpfXpavPCpZk2WJsBK92UNsD9jd+zWn+ytnKXAwIu/Tzo+siM3v9VNgirxeMs/\r\nOafuz2VNlFPVlH/cOSwJb7DoJw==\r\n-----END PRIVATE KEY-----\r\n".to_string(),
                m_pgp_public_key: "-----BEGIN PUBLIC KEY-----\r\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAuuL3bGb1GRPI9h0FMgN/\r\nK+FrwV72Rgm2sldXzJSQgny08gH8f94Y5aYDUPBooOtMFobc6dCab7nSQkS2XZjm\r\nveSMUHfNwnMNbqlvcd0qO5V4b9/EofmjxrIjQEyFeA9zK2xSt6Ngkomu8ALhqNtP\r\nX01QawJJmxvVwl7V+tAJcDaJfmctH6mSu32ZGtlTKrd5aD/AHiszPr2FhCp1AKr/\r\nCkGakuCA2NAQwspgH/+ZtVk5t87cGrVWQh5X2UXPyeDcTf5HZXnJ341kIuxSU9YH\r\nDpR58aOIT2j/5cl8E4WzHi+lK8zkjqHy1F3lkBabwYMYccCOgyU3UEiwbDEcd9tO\r\nswIDAQAB\r\n-----END PUBLIC KEY-----\r\n".to_string(),
            },
            m_machine_alias: "Hu-node".to_string(),
            m_backer_detail: UnlockDocument {
                m_unlock_sets: vec![
                    UnlockSet {
                        m_signature_type: "Strict".to_string(),
                        m_signature_ver: "0.0.1".to_string(),
                        m_signature_sets: vec![
                            IndividualSignature {
                                m_signer_id: "0000001".to_string(),
                                m_signature_key: "02deda35eeb5bceef530573917f6332c794fac354dc6a45c5dc1086c2880ae6173".to_string(),
                                m_permitted_to_pledge: "N".to_string(),
                                m_permitted_to_delegate: "N".to_string(),
                                m_input_time_lock: 0.0,
                                m_input_time_lock_strickt: 0.0,
                                m_output_time_lock: 0.0,
                            },
                            IndividualSignature {
                                m_signer_id: "0000002".to_string(),
                                m_signature_key: "02d2df353fe8e306b0986c774c674b298c56c51e9de4200a142dccde65721282e1".to_string(),
                                m_permitted_to_pledge: "N".to_string(),
                                m_permitted_to_delegate: "N".to_string(),
                                m_input_time_lock: 0.0,
                                m_input_time_lock_strickt: 0.0,
                                m_output_time_lock: 0.0,
                            },
                        ],
                        m_merkle_proof: vec![
                            "r.leave_4".to_string(),
                            "l.e7720680189ed8851aaa6dac86b75265b5867a52484c2e2afbd8a02c4d6230b8".to_string(),
                        ],
                        m_left_hash: "".to_string(),
                        m_salt: "7784f23b605ba6a4".to_string(),
                    },
                    UnlockSet {
                        m_signature_type: "Strict".to_string(),
                        m_signature_ver: "0.0.1".to_string(),
                        m_signature_sets: vec![
                            IndividualSignature {
                                m_signer_id: "0000000".to_string(),
                                m_signature_key: "0343b174ea00715e0b6011ba6198b7223c49e29e1141ed08e2b17b197cbe4c9301".to_string(),
                                m_permitted_to_pledge: "Y".to_string(),
                                m_permitted_to_delegate: "Y".to_string(),
                                m_input_time_lock: 0.0,
                                m_input_time_lock_strickt: 0.0,
                                m_output_time_lock: 0.0,
                            },
                            IndividualSignature {
                                m_signer_id: "0000001".to_string(),
                                m_signature_key: "02deda35eeb5bceef530573917f6332c794fac354dc6a45c5dc1086c2880ae6173".to_string(),
                                m_permitted_to_pledge: "N".to_string(),
                                m_permitted_to_delegate: "N".to_string(),
                                m_input_time_lock: 0.0,
                                m_input_time_lock_strickt: 0.0,
                                m_output_time_lock: 0.0,
                            },
                        ],
                        m_merkle_proof: vec![
                            "r.4bd81f336636a93f9ff6b17182388605ed3580b6f0afce81a8cb127d7182ce8c".to_string(),
                            "r.88b18ac38e17e322599e0cc304e3b493b25b28b97ae34c4f7dcd2dbf1478ae0a".to_string(),
                        ],
                        m_left_hash: "".to_string(),
                        m_salt: "f51ccac90852581a".to_string(),
                    },
                    UnlockSet {
                        m_signature_type: "Strict".to_string(),
                        m_signature_ver: "0.0.1".to_string(),
                        m_signature_sets: vec![
                            IndividualSignature {
                                m_signer_id: "0000000".to_string(),
                                m_signature_key: "0343b174ea00715e0b6011ba6198b7223c49e29e1141ed08e2b17b197cbe4c9301".to_string(),
                                m_permitted_to_pledge: "Y".to_string(),
                                m_permitted_to_delegate: "Y".to_string(),
                                m_input_time_lock: 0.0,
                                m_input_time_lock_strickt: 0.0,
                                m_output_time_lock: 0.0,
                            },
                            IndividualSignature {
                                m_signer_id: "0000002".to_string(),
                                m_signature_key: "02d2df353fe8e306b0986c774c674b298c56c51e9de4200a142dccde65721282e1".to_string(),
                                m_permitted_to_pledge: "N".to_string(),
                                m_permitted_to_delegate: "N".to_string(),
                                m_input_time_lock: 0.0,
                                m_input_time_lock_strickt: 0.0,
                                m_output_time_lock: 0.0,
                            },
                        ],
                        m_merkle_proof: vec!["r.88b18ac38e17e322599e0cc304e3b493b25b28b97ae34c4f7dcd2dbf1478ae0a".to_string()],
                        m_left_hash: "6288cf5ad27c2a5e00167b035152e97a8e77759db9a0ddeea074fbd0571e19eb".to_string(),
                        m_salt: "5b63e243dca37c8c".to_string(),
                    },
                ],
                m_merkle_root: "38a2d19509e4c4cb761a5c5a39d8d4153a0c28dc77ff4c07b34aa42e150de393".to_string(),
                m_account_address: "im1xqenscfjvscnjdfs89jngce5vd3rwd33vy6kxdtpxvukgwryxscs6xpe6r".to_string(),
                m_merkle_version: "0.1.0".to_string(),
                m_private_keys: HashMap::from([
                    (
                        "f51ccac90852581a".to_string(),
                        vec![
                            "4420f136fc2810907a9670bbab4d26b36ee6a02b759496f86fa0d3aed161a902".to_string(),
                            "b093b11f9412bb2e0acee606f3e12df6fe693fef3dce63392f3f26636b1a43ce".to_string(),
                        ]
                    ),
                    (
                        "7784f23b605ba6a4".to_string(),
                        vec![
                            "b093b11f9412bb2e0acee606f3e12df6fe693fef3dce63392f3f26636b1a43ce".to_string(),
                            "f5d31415c8ea9faf21f2742b8db7eec4906c3ce1f9c6b83d895db9e5f835231d".to_string(),
                        ]
                    ),
                    (
                        "5b63e243dca37c8c".to_string(),
                        vec![
                            "4420f136fc2810907a9670bbab4d26b36ee6a02b759496f86fa0d3aed161a902".to_string(),
                            "f5d31415c8ea9faf21f2742b8db7eec4906c3ce1f9c6b83d895db9e5f835231d".to_string(),
                        ]
                    ),
                ]),
            },
            m_language: "eng".to_string(),
            m_term_of_services: "Y".to_string(),
            m_already_presented_neighbors: vec![],
        },
    };
    return profile;
}