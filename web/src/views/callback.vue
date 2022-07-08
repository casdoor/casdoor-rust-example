<!--Copyright 2022 The Casdoor Authors. All Rights Reserved.-->

<!--Licensed under the Apache License, Version 2.0 (the "License");-->
<!--you may not use this file except in compliance with the License.-->
<!--You may obtain a copy of the License at-->

<!--     http://www.apache.org/licenses/LICENSE-2.0-->

<!--Unless required by applicable law or agreed to in writing, software-->
<!--distributed under the License is distributed on an "AS IS" BASIS,-->
<!--WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.-->
<!--See the License for the specific language governing permissions and-->
<!--limitations under the License.-->

<template>
  <h1>Callback</h1>
</template>

<script>
import * as config from "@/config";

export default {
  name: "callbackPage",
  mounted() {
    let code = this.getCode();
    console.log("code: " + code);
    this.getUserInfo(code);
  },
  methods: {
    // get url params
    getCode() {
      let url = window.location.href
      let params = url.split('?')[1]
      // get code from url
      for (let element of params.split('&')) {
        if (element.split('=')[0] == "code") {
          return element.split('=')[1]
        }
      }
        
      return "";
    },
    getUserInfo(code) {
      fetch(`${config.serverUrl}/auth/${code}`, {
        method: 'GET',
      }).then(res => {
        res.json().then(data => {
          localStorage.setItem('user', JSON.stringify(data));
          window.location.href = "/home";
        });
      });
    }, 
  }
}
</script>

<style scoped>


</style>
