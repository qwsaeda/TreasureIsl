/* ea:   0x0003DDF4 */
/* name: sub_3DDF4 */
/* size: 412 */
int __fastcall sub_3DDF4(int a1, int a2)
{
  _DWORD *v3; // r0
  _DWORD *v4; // r6
  int result; // r0
  _DWORD *v6; // r0
  _DWORD *v7; // r6
  int v8; // r4
  int v9; // r5
  _DWORD *v10; // r0
  _DWORD *v11; // r6
  int v12; // r4
  int v13; // r5
  void *v14; // r0
  void *v15; // r0
  void *v16; // r0

  switch ( a2 )
  {
    case 2:
      v3 = operator new(0x38u);
      v4 = v3;
      if ( v3 == nullptr )
        goto LABEL_14;
      sub_4EC88(v3);
      v4[12] = 0;
      *v4 = off_6F434;
      v4[13] = 0;
      result = (int)v4;
      break;
    case 4:
      v6 = operator new(0x48u);
      v7 = v6;
      if ( v6 == nullptr )
        goto LABEL_14;
      v8 = *(_DWORD *)(a1 + 20);
      v9 = *(_DWORD *)(a1 + 24);
      sub_4EC88(v6);
      v7[12] = 0;
      *v7 = off_6F410;
      v7[13] = 0;
      v7[14] = 220;
      v7[15] = 0;
      v7[16] = v8;
      v7[17] = v9;
      result = (int)v7;
      break;
    case 6:
      v10 = operator new(0x48u);
      v11 = v10;
      if ( v10 == nullptr )
        goto LABEL_14;
      v12 = *(_DWORD *)(a1 + 20);
      v13 = *(_DWORD *)(a1 + 24);
      sub_4EC88(v10);
      v11[12] = 0;
      *v11 = off_6F3EC;
      v11[13] = 0;
      v11[14] = 0;
      v11[15] = 0;
      v11[16] = v12;
      v11[17] = v13;
      result = (int)v11;
      break;
    case 8:
      v14 = operator new(0x54u);
      if ( v14 == nullptr )
        goto LABEL_14;
      result = sub_3EC44(v14, *(_DWORD *)(a1 + 20), *(_DWORD *)(a1 + 24));
      break;
    case 10:
      v15 = operator new(0x68u);
      if ( v15 == nullptr )
        goto LABEL_14;
      result = sub_401F0(v15, *(_DWORD *)(a1 + 20), *(_DWORD *)(a1 + 24));
      break;
    case 12:
    case 14:
      v16 = operator new(0x2ECu);
      if ( v16 == nullptr )
        goto LABEL_14;
      result = sub_1EECC((int)v16, *(_DWORD *)(a1 + 20), *(_DWORD *)(a1 + 24));
      break;
    default:
LABEL_14:
      result = 0;
      break;
  }
  return result;
}

