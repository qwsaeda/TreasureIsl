/* ea:   0x0003F4E4 */
/* name: sub_3F4E4 */
/* size: 1048 */
int __fastcall sub_3F4E4(int result, int a2, int a3)
{
  int *v3; // r6
  void (__fastcall ***v5)(int, int); // r3
  int v6; // r0
  int v7; // r0
  int v8; // r0
  __int16 **v9; // r0
  __int16 **v10; // r0
  wchar_t *v11; // r0
  int v12; // r3
  _DWORD *v13; // r4
  _DWORD *v14; // r0
  int v15; // r0
  int v16; // r1
  _DWORD v17[4]; // [sp+14h] [bp-7Ch] BYREF
  _DWORD v18[4]; // [sp+24h] [bp-6Ch] BYREF
  _DWORD v19[4]; // [sp+34h] [bp-5Ch] BYREF
  void *v20[4]; // [sp+44h] [bp-4Ch] BYREF
  void *v21[4]; // [sp+54h] [bp-3Ch] BYREF
  int v22[4]; // [sp+64h] [bp-2Ch] BYREF
  int v23; // [sp+74h] [bp-1Ch] BYREF

  v3 = (int *)result;
  if ( a2 != 0 )
    return result;
  if ( *(_DWORD *)(result + 72) != 0 )
  {
    (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(result + 72) + 16))(*(_DWORD *)(result + 72), 0);
    result = *(_DWORD *)(v3[18] + 1008);
    if ( result != 0 )
    {
      sub_17C7C(v3[1], 0);
      (*(void (__fastcall **)(int))(*(_DWORD *)v3[18] + 8))(v3[18]);
      v5 = (void (__fastcall ***)(int, int))v3[18];
      if ( v5 != nullptr )
        (**v5)(v3[18], 1);
      v3[18] = 0;
      return sub_3DC04(v3[1], 10);
    }
    return result;
  }
  sub_1333C(v17);
  sub_1333C(v19);
  sub_1333C(v18);
  if ( a3 > 38 )
  {
    if ( a3 == 39 )
    {
      v16 = v3[13];
      v7 = 1;
      if ( v16 != v3[16] )
        v7 = v16 + 1;
      goto LABEL_37;
    }
    if ( a3 != 40 )
      goto LABEL_38;
    v15 = v3[13];
    if ( v15 == 5 )
    {
      v7 = 3;
      goto LABEL_37;
    }
    if ( v15 >= 3 )
      goto LABEL_27;
LABEL_33:
    v7 = v15 + 2;
    goto LABEL_37;
  }
  if ( a3 != 38 )
  {
    if ( a3 == 13 )
    {
      sub_17C7C(v3[1], 0);
      dword_7FB70 = v3[13];
      sub_1309C((int)v17);
      v8 = sub_12F04(aData);
      sub_134A4((int)v17, (int)aData, (int)&aData[v8]);
      v9 = (__int16 **)sub_37EDC(v20, (int)v17, (char *)aUiPak);
      if ( v9 != v19 )
        sub_17890((int)v19, *v9, v9[1]);
      sub_132A0(v20);
      v10 = (__int16 **)sub_37EDC(v21, (int)v17, (char *)aHelpMap);
      if ( v10 != v18 )
        sub_17890((int)v18, *v10, v10[1]);
      sub_132A0(v21);
      v11 = (wchar_t *)operator new(0x830u);
      if ( v11 != nullptr )
        v12 = sub_3F8FC(v11);
      else
        v12 = 0;
      v3[18] = v12;
      if ( v12 != 0 )
      {
        sub_43D54(v12, *(_DWORD *)(v3[1] + 16));
        v13 = sub_132FC(v22, v18);
        v14 = sub_132FC(&v23, v19);
        (*(void (__fastcall **)(int, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)v3[18] + 4))(
          v3[18],
          *v14,
          v14[1],
          v14[2],
          *v13,
          v13[1],
          v13[2]);
      }
      goto LABEL_38;
    }
    if ( a3 == 27 )
    {
      sub_17C7C(v3[1], 0);
      sub_3DC04(v3[1], 6);
      goto LABEL_38;
    }
    if ( a3 != 37 )
      goto LABEL_38;
    v6 = v3[13];
    if ( v6 == 1 )
      v7 = v3[16];
    else
      v7 = v6 - 1;
    goto LABEL_37;
  }
  v15 = v3[13];
  if ( v15 == 5 )
  {
    v7 = 1;
    goto LABEL_37;
  }
  if ( v15 <= 2 )
    goto LABEL_33;
LABEL_27:
  v7 = v15 - 2;
LABEL_37:
  v3[13] = v7;
LABEL_38:
  switch ( v3[13] )
  {
    case 1:
      v3[14] = 18;
      v3[15] = 11;
      sub_12EA0((int)v18);
      sub_12EA0((int)v19);
      result = sub_12EA0((int)v17);
      break;
    case 2:
      v3[14] = 101;
      v3[15] = 11;
      sub_12EA0((int)v18);
      sub_12EA0((int)v19);
      result = sub_12EA0((int)v17);
      break;
    case 3:
      v3[14] = 18;
      v3[15] = 115;
      sub_12EA0((int)v18);
      sub_12EA0((int)v19);
      result = sub_12EA0((int)v17);
      break;
    case 4:
      v3[14] = 101;
      v3[15] = 115;
      sub_12EA0((int)v18);
      sub_12EA0((int)v19);
      result = sub_12EA0((int)v17);
      break;
    case 5:
      v3[14] = 58;
      v3[15] = 75;
      goto LABEL_44;
    default:
LABEL_44:
      sub_12EA0((int)v18);
      sub_12EA0((int)v19);
      result = sub_12EA0((int)v17);
      break;
  }
  return result;
}

