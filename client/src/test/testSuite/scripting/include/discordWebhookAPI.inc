#include <ripext>

methodmap EmbedFooter < JSONObject
{
	/**
	 * Constructor for the EmbedFooter methodmap.
	 * 
	 * @param text     Text of the footer.
	 * @return         Returns the EmbedFooter.
	 */
	public EmbedFooter(const char[] text="") 
	{
		JSONObject jsonObject = new JSONObject();
		jsonObject.SetString("text", text);
		return view_as<EmbedFooter>(jsonObject);
	}

	/**
	 * Retrieve the text of the footer.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetText(char[] buffer, int maxlength)
	{
		return this.GetString("text", buffer, maxlength);
	}

	/**
	 * Set the text of the footer.
	 * 
	 * @param text   				Text of the footer.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetText(const char[] text)
	{
		return this.SetString("text", text);
	}

	/**
	 * Retrieve the url of the footer icon.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetIconURL(char[] buffer, int maxlength)
	{
		return this.GetString("icon_url", buffer, maxlength);
	}

	/**
	 * Set the url of the footer icon.
	 * Note : this only supports http(s).
	 * 
	 * @param icon_url   		IconURL of the footer.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetIconURL(const char[] icon_url)
	{
		return this.SetString("icon_url", icon_url);
	}

	/**
	 * Retrieve the proxy_icon_url of the footer.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetProxyIconURL(char[] buffer, int maxlength)
	{
		return this.GetString("proxy_icon_url", buffer, maxlength);
	}

	/**
	 * Set a proxied url of the footer icon.
	 * 
	 * @param proxy_icon_url   		IconURL of the footer.
	 * @return        						True on success. False otherwise.
	 */
	public bool SetProxyIconURL(const char[] proxy_icon_url)
	{
		return this.SetString("proxy_icon_url", proxy_icon_url);
	}
}


methodmap EmbedImage < JSONObject
{
	/**
	 * Constructor for the EmbedImage methodmap.
	 * 
	 * @param url     URL of the image.
	 * @return         Returns the EmbedImage.
	 */
	public EmbedImage(const char[] url="") 
	{
		JSONObject jsonObject = new JSONObject();
		jsonObject.SetString("url", url);
		return view_as<EmbedImage>(jsonObject);
	}

	/**
	 * Retrieve the source url of the media.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetURL(char[] buffer, int maxlength)
	{
		return this.GetString("url", buffer, maxlength);
	}

	/**
	 * Set the source url of the media.
	 * Note : this only supports http(s).
	 * 
	 * @param url   		URL of the media.
	 * @return        	True on success. False otherwise.
	 */
	public bool SetURL(const char[] url)
	{
		return this.SetString("url", url);
	}

	/**
	 * Retrieve the proxied URL of the media.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetProxyURL(char[] buffer, int maxlength)
	{
		return this.GetString("proxy_url", buffer, maxlength);
	}

	/**
	 * Set a proxied URL of the media.
	 * 
	 * @param proxy_url   		Proxy URL of the media.
	 * @return        				True on success. False otherwise.
	 */
	public bool SetProxyURL(const char[] proxy_url)
	{
		return this.SetString("proxy_url", proxy_url);
	}

	/**
	 * Retrieve the height of the media.
	 * 
	 * @return    Int value of the media.
	 */
	public int GetHeight()
	{
		return this.GetInt("height");
	}

	/**
	 * Set the height of the media.
	 * 
	 * @param height   	Height of the media.
	 * @return        	True on success. False otherwise.
	 */
	public bool SetHeight(int height)
	{
		return this.SetInt("height", height);
	}

	/**
	 * Retrieve the width of the media.
	 * 
	 * @return    Int value of the media.
	 */
	public int GetWidth()
	{
		return this.GetInt("width");
	}

	/**
	 * Set the width of the media.
	 * 
	 * @param width   	Width of the media.
	 * @return        	True on success. False otherwise.
	 */
	public bool SetWidth(int width)
	{
		return this.SetInt("width", width);
	}
}


methodmap EmbedThumbnail < EmbedImage
{
	/**
	 * Constructor for the EmbedThumbnail methodmap.
	 * 
	 * @param url     URL of the thumbnail.
	 * @return         Returns the EmbedThumbnail.
	 */
	public EmbedThumbnail(const char[] url="") 
	{
		JSONObject jsonObject = new JSONObject();
		jsonObject.SetString("url", url);
		return view_as<EmbedThumbnail>(jsonObject);
	}
}


methodmap EmbedVideo < EmbedImage
{
	/**
	 * Constructor for the EmbedVideo methodmap.
	 * 
	 * @param url      URL of the video.
	 * @return         Returns the EmbedVideo.
	 */
	public EmbedVideo(const char[] url="") 
	{
		JSONObject jsonObject = new JSONObject();
		jsonObject.SetString("url", url);
		return view_as<EmbedVideo>(jsonObject);
	}
}


methodmap EmbedProvider < JSONObject
{
	/**
	 * Constructor for the EmbedProvider methodmap.
	 * 
	 * @param name     Name of the provider.
	 * @return         Returns the EmbedProvider.
	 */
	public EmbedProvider(const char[] name="") 
	{
		JSONObject jsonObject = new JSONObject();
		jsonObject.SetString("name", name);
		return view_as<EmbedProvider>(jsonObject);
	}

	/**
	 * Retrieve the name of the provider.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetName(char[] buffer, int maxlength)
	{
		return this.GetString("name", buffer, maxlength);
	}

	/**
	 * Set the name of the provider.
	 * 
	 * @param name   		Name of the provider.
	 * @return        	True on success. False otherwise.
	 */
	public bool SetName(const char[] name)
	{
		return this.SetString("name", name);
	}

		/**
	 * Retrieve the source url of the provider.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetURL(char[] buffer, int maxlength)
	{
		return this.GetString("url", buffer, maxlength);
	}

	/**
	 * Set the source url of the provider.
	 * 
	 * @param url   		URL of the provider.
	 * @return        	True on success. False otherwise.
	 */
	public bool SetURL(const char[] url)
	{
		return this.SetString("url", url);
	}
}


methodmap EmbedAuthor < JSONObject
{
	/**
	 * Constructor for the EmbedAuthor methodmap.
	 * 
	 * @param name     Name of the author.
	 * @return         Returns the EmbedAuthor.
	 */
	public EmbedAuthor(const char[] name="") 
	{
		JSONObject jsonObject = new JSONObject();
		jsonObject.SetString("name", name);
		return view_as<EmbedAuthor>(jsonObject);
	}

	/**
	 * Retrieve the name of the author.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetName(char[] buffer, int maxlength)
	{
		return this.GetString("name", buffer, maxlength);
	}

	/**
	 * Set the name of the author.
	 * 
	 * @param name   				Name of the author.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetName(const char[] name)
	{
		return this.SetString("name", name);
	}

	/**
	 * Retrieve the url of the author.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetURL(char[] buffer, int maxlength)
	{
		return this.GetString("url", buffer, maxlength);
	}

	/**
	 * Set the url of the author.
	 * 
	 * @param url   	URL of the author.
	 * @return        True on success. False otherwise.
	 */
	public bool SetURL(const char[] url)
	{
		return this.SetString("url", url);
	}

	/**
	 * Retrieve the icon_url of the author.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetIconURL(char[] buffer, int maxlength)
	{
		return this.GetString("icon_url", buffer, maxlength);
	}

	/**
	 * Set the icon_url of the author.
	 * Note : this only supports http(s).
	 * 
	 * @param icon_url   		IconURL of the author.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetIconURL(const char[] icon_url)
	{
		return this.SetString("icon_url", icon_url);
	}

	/**
	 * Retrieve the proxy_icon_url of the author.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetProxyIconURL(char[] buffer, int maxlength)
	{
		return this.GetString("proxy_icon_url", buffer, maxlength);
	}

	/**
	 * Set a proxied url of the author.
	 * 
	 * @param proxy_icon_url   		IconURL of the author.
	 * @return        						True on success. False otherwise.
	 */
	public bool SetProxyIconURL(const char[] proxy_icon_url)
	{
		return this.SetString("proxy_icon_url", proxy_icon_url);
	}
}


methodmap EmbedField < JSONObject
{
	/**
	 * Constructor for the EmbedField methodmap.
	 * 
	 * @param name     Name of the field.
	 * @param value    Value of the field.
	 * @param inline   Bool value of the inline @parameter.
	 * @return         Returns the EmbedField.
	 */
	public EmbedField(const char[] name="", const char[] value="", bool inline=true) 
	{
		JSONObject jsonObject = new JSONObject();
		jsonObject.SetString("name", name);
		jsonObject.SetString("value", value);
		jsonObject.SetBool("inline", inline);
		return view_as<EmbedField>(jsonObject);
	}

	/**
	 * Retrieve the name of the field.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetName(char[] buffer, int maxlength)
	{
		return this.GetString("name", buffer, maxlength);
	}

	/**
	 * Set the name of the field.
	 * 
	 * @param name   				Name of the field.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetName(const char[] name)
	{
		return this.SetString("name", name);
	}

	/**
	 * Retrieve the value of the field.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetValue(char[] buffer, int maxlength)
	{
		return this.GetString("value", buffer, maxlength);
	}

	/**
	 * Set the value of the field.
	 * 
	 * @param value   	Value of the field.
	 * @return        	True on success. False otherwise.
	 */
	public bool SetValue(const char[] value)
	{
		return this.SetString("value", value);
	}

	/**
	 * Retrieve the inline @parameter of the field.
	 * 
	 * @return			The bool value of the inline @parameter.
	 */
	public bool GetInline()
	{
		return this.GetBool("inline");
	}

	/**
	 * Set whether or not this field should display inline.
	 * 
	 * @param inline   			Bool value of the inline @parameter.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetInline(bool inline)
	{
		return this.SetBool("inline", inline);
	}
}

methodmap Embed < JSONObject
{
	/**
	 * Constructor for the Embed methodmap.
	 * 
	 * @param title          Title of the embed.
	 * @param description    Description of the embed.
	 * @return               Returns the Embed.
	 */
	public Embed(const char[] title="", const char[] description="") 
	{
		JSONObject jsonObject = new JSONObject();
		jsonObject.SetInt("color", 0);
		jsonObject.SetString("title", title);
		jsonObject.SetString("description", description);
		return view_as<Embed>(jsonObject);
	}

	/**
	 * Retrieve the title of the embed.
	 * 
	 * @param szBuffer        String buffer to store value.
	 * @param iBufferSize     Maximum length of the string buffer.
	 * @return                True on success. False otherwise.
	 */
	public bool GetTitle(char[] buffer, int maxlength)
	{
		return this.GetString("title", buffer, maxlength);
	}

	/**
	 * Set the title of the embed.
	 * 
	 * @param title     Title of the embed.
	 * @return          True on success. False otherwise.
	 */
	public bool SetTitle(const char[] title)
	{
		return this.SetString("title", title);
	}

	/**
	 * Retrieve the description of the embed.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetDescription(char[] buffer, int maxlength)
	{
		return this.GetString("description", buffer, maxlength);
	}

	/**
	 * Set the description of the embed.
	 * 
	 * @param description   Description of the embed.
	 * @return          		True on success. False otherwise.
	 */
	public bool SetDescription(const char[] description)
	{
		return this.SetString("description", description);
	}

	/**
	 * Retrieve the URL of the embed.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetURL(char[] buffer, int maxlength)
	{
		return this.GetString("url", buffer, maxlength);
	}

	/**
	 * Set the URL of the embed.
	 * 
	 * @param url   	URL of the embed.
	 * @return        True on success. False otherwise.
	 */
	public bool SetURL(const char[] url)
	{
		return this.SetString("url", url);
	}

	/**
	 * Retrieve the timestamp of the embed.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetTimeStamp(char[] buffer, int maxlength)
	{
		return this.GetString("timestamp", buffer, maxlength);
	}

	/**
	 * Set the timestamp of the embed.
	 * Note that the webhook will error out if the timestamp is not ISO 8601 complient.
	 * Example: 1977-04-22T06:00:00Z
	 * Use FormatTime to format it properly, see https://github.com/Sarrus1/discordWebhookAPI/blob/master/example.sp for an example.
	 * 
	 * @param timestamp   	Timestamp of the embed.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetTimeStamp(const char[] timestamp)
	{
		return this.SetString("timestamp", timestamp);
	}

	/**
	 * Set the timestamp of the embed to now.
	 * 
	 * @return        			True on success. False otherwise.
	 */
	public bool SetTimeStampNow()
	{
		char timeNow[256];
		FormatTime(timeNow, sizeof timeNow, "%FT\%T.000%z", GetTime());
		return this.SetString("timestamp", timeNow);
	}

	/**
	 * Retrieve the timestamp of the embed.
	 * 
	 * @return    Int value of the color.
	 */
	public int GetColor()
	{
		return this.GetInt("color");
	}

	/**
	 * Set the color of the embed.
	 * 
	 * @param color   	Color code of the embed.
	 * @return        	True on success. False otherwise.
	 */
	public bool SetColor(int color)
	{
		return this.SetInt("color", color);
	}

	/**
	 * Retrieve the footer of the embed.
	 * 
	 * @return    Footer of the embed.
	 */
	public EmbedFooter GetFooter()
	{
		return view_as<EmbedFooter>(this.Get("footer"));
	}

	/**
	 * Set the footer of the embed.
	 * 
	 * @param footer   	Footer of the embed.
	 * @return        	True on success. False otherwise.
	 */
	public bool SetFooter(EmbedFooter footer)
	{
		return this.Set("footer", view_as<JSON>(footer));
	}

	/**
	 * Retrieve the image of the embed.
	 * 
	 * @return    Image of the embed.
	 */
	public EmbedImage GetImage()
	{
		return view_as<EmbedImage>(this.Get("image"));
	}

	/**
	 * Set the image of the embed.
	 * 
	 * @param image   	Image of the embed.
	 * @return        	True on success. False otherwise.
	 */
	public bool SetImage(EmbedImage image)
	{
		return this.Set("image", view_as<JSON>(image));
	}

	/**
	 * Retrieve the thumbnail of the embed.
	 * 
	 * @return    Thumbnail of the embed.
	 */
	public EmbedThumbnail GetThumbnail()
	{
		return view_as<EmbedThumbnail>(this.Get("thumbnail"));
	}

	/**
	 * Set the thumbnail of the embed.
	 * 
	 * @param thumbnail   	Thumbnail of the embed.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetThumbnail(EmbedThumbnail thumbnail)
	{
		return this.Set("thumbnail", view_as<JSON>(thumbnail));
	}

	/**
	 * Retrieve the video of the embed.
	 * 
	 * @return    Video of the embed.
	 */
	public EmbedVideo GetVideo()
	{
		return view_as<EmbedVideo>(this.Get("video"));
	}

	/**
	 * Set the video of the embed.
	 * 
	 * @param video   			Video of the embed.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetVideo(EmbedVideo video)
	{
		return this.Set("video", view_as<JSON>(video));
	}

	/**
	 * Retrieve the provider of the embed.
	 * 
	 * @return    Provider of the embed.
	 */
	public EmbedProvider GetProvider()
	{
		return view_as<EmbedProvider>(this.Get("provider"));
	}

	/**
	 * Set the provider of the embed.
	 * 
	 * @param provider   			Provider of the embed.
	 * @return        				True on success. False otherwise.
	 */
	public bool SetProvider(EmbedProvider provider)
	{
		return this.Set("provider", view_as<JSON>(provider));
	}

	/**
	 * Retrieve the author of the embed.
	 * 
	 * @return    Author of the embed.
	 */
	public EmbedAuthor GetAuthor()
	{
		return view_as<EmbedAuthor>(this.Get("author"));
	}

	/**
	 * Set the author of the embed.
	 * 
	 * @param author   			Author of the embed.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetAuthor(EmbedAuthor author)
	{
		return this.Set("author", view_as<JSON>(author));
	}

	/**
	 * Retrieve the fields of the embed.
	 * 
	 * @return    JSONArray containing the fields of the embed from its index.
	 */
	public JSONArray GetFields()
	{
		return view_as<JSONArray>(this.Get("fields"));
	}

	/**
	 * Retrieve a field of the embed.
	 * 
	 * @return    Field corresponding to the input index. null if an error occurs.
	 */
	public EmbedField GetField(int index)
	{
		JSONArray fields = view_as<JSONArray>(this.Get("fields"));
		if(fields != null && fields.Length < index)
		{
			return view_as<EmbedField>(fields.Get(index));
		}
		return null;
	}

	/**
	 * Add a field to the embed. This will delete the field.
	 * 
	 * @param field   			Field to add to the embed.
	 * @return        			The number of fields after the new one was added. -1 otherwise.
	 */
	public int AddField(EmbedField field)
	{
		JSONArray fields;
		if(this.HasKey("fields"))
		{
			fields = view_as<JSONArray>(this.Get("fields"));
		}
		else
		{
			fields = new JSONArray();
		}
		if(fields.Push(view_as<JSON>(field)))
		{
			if(this.Set("fields", fields))
			{
				delete field;
				return fields.Length;
			}
		}
		return -1;
	}
}


methodmap Webhook < JSONObject
{
	/**
	 * Constructor for the Embed methodmap.
	 * 
	 * @param content   Content of the webhook.
	 * @return         Returns the Embed.
	 */
	public Webhook(const char[] content="") 
	{
		JSONObject jsonObject = new JSONObject();
		jsonObject.SetString("content", content);
		return view_as<Webhook>(jsonObject);
	}

	/**
	 * Retrieve the content of the webhook.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetContent(char[] buffer, int maxlength)
	{
		return this.GetString("content", buffer, maxlength);
	}

	/**
	 * Set the content of the webhook.
	 * 
	 * @param content   		Content of the webhook.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetContent(const char[] content)
	{
		return this.SetString("content", content);
	}

	/**
	 * Retrieve the username of the webhook.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetUsername(char[] buffer, int maxlength)
	{
		return this.GetString("username", buffer, maxlength);
	}

	/**
	 * Set the username of the webhook.
	 * 
	 * @param username   		Username of the webhook.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetUsername(const char[] username)
	{
		return this.SetString("username", username);
	}

	/**
	 * Retrieve the avatar_url of the webhook.
	 * 
	 * @param buffer        String buffer to store value.
	 * @param maxlength     Maximum length of the string buffer.
	 * @return              True on success. False otherwise.
	 */
	public bool GetAvatarURL(char[] buffer, int maxlength)
	{
		return this.GetString("avatar_url", buffer, maxlength);
	}

	/**
	 * Set the avatar URL of the webhook.
	 * 
	 * @param avatar_url   	Avatar URL of the webhook.
	 * @return        			True on success. False otherwise.
	 */
	public bool SetAvatarURL(const char[] avatar_url)
	{
		return this.SetString("avatar_url", avatar_url);
	}

	/**
	 * Retrieve the tts @parameter of the webhook.
	 * 
	 * @return    Bool representing the tts @parameter of the webhook.
	 */
	public bool GetTTS()
	{
		return this.GetBool("tts");
	}

	/**
	 * Set the tts @parameter of the webhook.
	 * 
	 * @param tts   		TTS @parameter of the webhook.
	 * @return        	True on success. False otherwise.
	 */
	public bool SetTTS(bool tts)
	{
		return this.SetBool("tts", tts);
	}

	/**
	 * Retrieve the embeds of the webhook.
	 * 
	 * @return    JSONArray containing the embeds of the webhook.
	 */
	public JSONArray GetEmbeds()
	{
		return view_as<JSONArray>(this.Get("embeds"));
	}

	/**
	 * Retrieve an embed of the webook from its index.
	 * 
	 * @return    Embed corresponding to the input index. null if an error occurs.
	 */
	public Embed GetEmbed(int index)
	{
		JSONArray embeds = view_as<JSONArray>(this.Get("embeds"));
		if(embeds != null && embeds.Length < index)
		{
			return view_as<Embed>(embeds.Get(index));
		}
		return null;
	}

	/**
	 * Add an embed to the webhook. This will delete the handle to the embed.
	 * 
	 * @param embed   			Embed to add to the webhook.
	 * @return        			The number of embeds after the new one was added. -1 otherwise.
	 */
	public int AddEmbed(Embed embed)
	{
		JSONArray embeds;
		if(this.HasKey("embeds"))
		{
			embeds = view_as<JSONArray>(this.Get("embeds"));
		}
		else
		{
			embeds = new JSONArray();
		}
		if(embeds.Push(view_as<JSON>(embed)))
		{
			if(this.Set("embeds", embeds))
			{
				delete embed;
				return embeds.Length;
			}
		}
		return -1;
	}

	/**
	 * Execute the webhook
	 * 
	 * @param webhook      URL of the webhook.
	 * @param callback     Callback function called when the HTTP request has been processed.
	 * @param data         Optional value to pass to the callback function.
	 */
	public void Execute(const char[] webhook, HTTPRequestCallback callback, any data = 0)
	{
		HTTPRequest httpRequest = new HTTPRequest(webhook);
		httpRequest.Post(view_as<JSON>(this), callback, data);
		return;
	}
}