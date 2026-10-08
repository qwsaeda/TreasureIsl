/* ea:   0x0001DB10 */
/* name: sub_1DB10 */
/* size: 476 */
BOOL __fastcall sub_1DB10(_DWORD *a1)
{
  int v2; // r9
  int v3; // r5
  _DWORD *i; // r8
  BOOL result; // r0
  int v6; // r1
  int v7; // r0
  int v8; // r6
  int v9; // r4
  int v10; // r0
  int v11; // r1
  int v12; // r2
  int v13; // r0
  _DWORD *v14; // r2
  int v15; // t1
  int v16; // r1
  int (*v17)(); // r0
  int v18; // r4
  _BYTE v19[8]; // [sp+Ch] [bp-4Ch] BYREF
  _DWORD v20[4]; // [sp+14h] [bp-44h] BYREF
  _DWORD v21[13]; // [sp+24h] [bp-34h] BYREF

  v2 = 0;
  v3 = a1[12] + 508;
  for ( i = *(_DWORD **)sub_1DDF0(v3, v19); ; i = sub_16A30(i) )
  {
    result = i != *(_DWORD **)sub_1DE08(v3, v19);
    if ( !result )
      break;
    v6 = i[5];
    if ( *(_BYTE *)(v6 + 50) == 0 )
      continue;
    v7 = *(_DWORD *)(v6 + 12) + a1[14];
    v8 = v7 + 50;
    v9 = *(_DWORD *)(v6 + 16) + a1[15];
    v21[1] = v7 + 34;
    v10 = a1[23];
    v21[0] = v8 - 32;
    v21[2] = v9 - 16;
    v21[3] = v9;
    switch ( v10 )
    {
      case 0:
        v2 = 0;
        v11 = sub_44F98(i[5], a1[12]);
        goto LABEL_10;
      case 1:
        v2 = 16;
        v11 = *(unsigned __int16 *)(i[5] + 22);
        goto LABEL_10;
      case 2:
        v2 = 32;
        v11 = *(unsigned __int16 *)(i[5] + 24);
LABEL_10:
        sub_1EA70(a1, v11, v8, v9);
        break;
      default:
        break;
    }
    v12 = a1[8];
    v20[0] = 0;
    v20[2] = v2;
    v20[1] = 16;
    v20[3] = v2 + 16;
    v15 = *(_DWORD *)(v12 + 12);
    v14 = (_DWORD *)(v12 + 12);
    v13 = v15;
    if ( v15 == 0 )
      __debugbreak();
    v16 = *(_DWORD *)(v13 + 12);
    if ( v16 == 16 )
    {
      v17 = sub_5BC14;
    }
    else if ( v16 == 8 )
    {
      v17 = sub_5D9C4;
    }
    else
    {
      v17 = nullptr;
    }
    v18 = a1[2];
    a1[3] = v17;
    (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v18 + 20))(
      v18,
      v21,
      *v14,
      v20,
      v17,
      &dword_81230,
      1);
  }
  return result;
}

