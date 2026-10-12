/* ea:   0x0001C524 */
/* name: sub_1C524 */
/* size: 756 */
int __fastcall sub_1C524(int a1, int a2)
{
  _WORD *v3; // r0
  unsigned __int16 v4; // r5
  unsigned __int16 v5; // r6
  unsigned __int16 v6; // r7
  unsigned __int16 v7; // r8
  int v8; // r0
  unsigned __int16 v9; // r3
  unsigned __int16 v10; // r1
  unsigned __int16 v11; // r3
  unsigned __int16 v12; // r0
  unsigned __int16 v13; // r0

  v3 = (_WORD *)(a1 + 1024);
  v4 = v3[10];
  v5 = v3[11];
  v6 = v3[6];
  v7 = *(_WORD *)(a1 + 1038);
  v8 = *(_DWORD *)(a1 + 1052);
  if ( a2 == 0 )
  {
    if ( v8 == 1 )
    {
      *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 302);
      v11 = v4;
      if ( v4 != 0 )
      {
        if ( v6 < 0x3E7u )
        {
          ++v6;
          --v4;
          if ( v5 > (unsigned int)(unsigned __int16)(v11 - 1) )
          {
            if ( v7 < 0x3E7u )
              ++v7;
            --v5;
          }
        }
      }
      else
      {
        v12 = v6;
        v6 = 0;
        v4 = v12;
      }
    }
    else if ( v8 == 3 )
    {
      *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 702);
      if ( v5 != 0 )
      {
        if ( v7 < 0x3E7u )
        {
          ++v7;
          --v5;
        }
      }
      else
      {
        v9 = v7;
        if ( v7 > (unsigned int)v4 )
          v7 = v4;
        v10 = v7;
        v7 = v9 - v7;
        v5 = v10;
      }
    }
    goto LABEL_32;
  }
  if ( v8 == 1 )
  {
    *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 402);
    if ( v6 == 0 )
    {
      v6 = v4;
      v4 = 0;
      if ( v5 == 0 )
        goto LABEL_32;
      if ( v5 + (unsigned int)v7 > 0x3E7 )
        v7 = 999;
      else
        v7 += v5;
      v13 = 0;
LABEL_31:
      v5 = v13;
      goto LABEL_32;
    }
    if ( v4 < 0x3E7u )
    {
      --v6;
      ++v4;
    }
  }
  else if ( v8 == 3 )
  {
    *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 802);
    if ( v7 == 0 )
    {
      v7 = v5;
      v5 = 0;
      goto LABEL_32;
    }
    if ( v5 < (unsigned int)v4 )
    {
      --v7;
      v13 = v5 + 1;
      goto LABEL_31;
    }
  }
LABEL_32:
  *(_WORD *)(a1 + 1044) = v4;
  *(_WORD *)(a1 + 1046) = v5;
  *(_WORD *)(a1 + 1036) = v6;
  *(_WORD *)(a1 + 1038) = v7;
  return 1036;
}

