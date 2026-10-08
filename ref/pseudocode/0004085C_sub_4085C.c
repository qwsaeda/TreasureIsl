/* ea:   0x0004085C */
/* name: sub_4085C */
/* size: 1032 */
int __fastcall sub_4085C(int result, int a2, int a3)
{
  int *v3; // r6
  void (__fastcall ***v5)(int, int); // r3
  int v6; // r0
  __int16 **v7; // r0
  __int16 **v8; // r0
  wchar_t *v9; // r0
  wchar_t *v10; // r3
  _DWORD *v11; // r4
  _DWORD *v12; // r0
  int v13; // r3
  int v14; // r0
  _DWORD v15[4]; // [sp+14h] [bp-7Ch] BYREF
  _DWORD v16[4]; // [sp+24h] [bp-6Ch] BYREF
  _DWORD v17[4]; // [sp+34h] [bp-5Ch] BYREF
  void *v18[4]; // [sp+44h] [bp-4Ch] BYREF
  void *v19[4]; // [sp+54h] [bp-3Ch] BYREF
  int v20[4]; // [sp+64h] [bp-2Ch] BYREF
  int v21; // [sp+74h] [bp-1Ch] BYREF

  v3 = (int *)result;
  if ( a2 != 0 )
    return result;
  if ( *(_DWORD *)(result + 92) == 0 )
  {
    sub_1333C(v15);
    sub_1333C(v17);
    sub_1333C(v16);
    if ( a3 > 38 )
    {
      if ( a3 != 39 )
      {
        if ( a3 == 40 )
        {
          v14 = v3[22];
          if ( v14 != 0 )
            v3[22] = v14 - 1;
          else
            v3[22] = ((v3[14] - v3[13]) >> 2) - 1;
        }
        goto LABEL_32;
      }
    }
    else
    {
      switch ( a3 )
      {
        case 38:
          v13 = v3[22];
          if ( v13 != ((v3[14] - v3[13]) >> 2) - 1 )
          {
            v3[22] = v13 + 1;
            sub_12EA0((int)v16);
            sub_12EA0((int)v17);
            return sub_12EA0((int)v15);
          }
          goto LABEL_31;
        case 13:
          sub_17C7C(v3[1], 0);
          dword_7FB6C = *(_DWORD *)(v3[13] + 4 * v3[22]);
          sub_1309C((int)v15);
          v6 = sub_12F04(aData);
          sub_134A4((int)v15, (int)aData, (int)&aData[v6]);
          v7 = (__int16 **)sub_37EDC(v18, (int)v15, (char *)aUiPak);
          if ( v7 != v17 )
            sub_17890((int)v17, *v7, v7[1]);
          sub_132A0(v18);
          v8 = (__int16 **)sub_37EDC(v19, (int)v15, (char *)aHelpMap);
          if ( v8 != v16 )
            sub_17890((int)v16, *v8, v8[1]);
          sub_12F48(v19);
          v9 = (wchar_t *)operator new(0x830u);
          if ( v9 != nullptr )
            v10 = sub_3F8FC(v9, v3[2], v3[25], 2, off_6F32C[v3[22]]);
          else
            v10 = nullptr;
          v3[23] = (int)v10;
          if ( v10 != nullptr )
          {
            sub_43D54(v10, *(_DWORD *)(v3[1] + 16));
            v11 = sub_132FC(v20, v16);
            v12 = sub_132FC(&v21, v17);
            (*(void (__fastcall **)(int, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)v3[23] + 4))(
              v3[23],
              *v12,
              v12[1],
              v12[2],
              *v11,
              v11[1],
              v11[2]);
            sub_12EA0((int)v16);
            sub_12EA0((int)v17);
            return sub_12EA0((int)v15);
          }
          goto LABEL_32;
        case 27:
          sub_17C7C(v3[1], 0);
          sub_3DC04(v3[1], 8);
          sub_12EA0((int)v16);
          sub_12EA0((int)v17);
          return sub_12EA0((int)v15);
        default:
          break;
      }
      if ( a3 != 37 )
      {
LABEL_32:
        sub_12EA0((int)v16);
        sub_12EA0((int)v17);
        return sub_12EA0((int)v15);
      }
    }
    if ( v3[22] < ((v3[14] - v3[13]) >> 2) - 1 )
    {
      v3[22] = ((v3[14] - v3[13]) >> 2) - 1;
      sub_12EA0((int)v16);
      sub_12EA0((int)v17);
      return sub_12EA0((int)v15);
    }
LABEL_31:
    v3[22] = 0;
    goto LABEL_32;
  }
  (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(result + 92) + 16))(*(_DWORD *)(result + 92), 0);
  result = *(_DWORD *)(v3[23] + 1008);
  if ( result != 0 )
  {
    sub_17C7C(v3[1], 0);
    (*(void (__fastcall **)(int))(*(_DWORD *)v3[23] + 8))(v3[23]);
    v5 = (void (__fastcall ***)(int, int))v3[23];
    if ( v5 != nullptr )
      (**v5)(v3[23], 1);
    v3[23] = 0;
    return sub_3DC04(v3[1], 12);
  }
  return result;
}

