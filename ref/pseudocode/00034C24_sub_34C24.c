/* ea:   0x00034C24 */
/* name: sub_34C24 */
/* size: 376 */
_DWORD *__fastcall sub_34C24(_DWORD *a1)
{
  _DWORD *result; // r0
  _DWORD *v3; // r10
  int (*v4)(); // r8
  int v5; // r6
  int v6; // r9
  int v7; // r7
  int v8; // r0
  int (**v9)(); // r0
  int v10; // r4
  int v11; // r6
  unsigned __int16 *v12; // r9
  int v13; // r8
  int v14; // r2
  int v15; // r7
  int v16; // r1
  unsigned int v17; // r5
  int v18; // r4
  int v19; // r0
  int v20; // t1
  int v21; // [sp+Ch] [bp-44h] BYREF
  int v22; // [sp+10h] [bp-40h]
  int v23; // [sp+14h] [bp-3Ch]
  int v24; // [sp+18h] [bp-38h]
  _DWORD v25[13]; // [sp+1Ch] [bp-34h] BYREF

  result = (_DWORD *)a1[183];
  v3 = result;
  if ( result == nullptr )
    return result;
  v4 = nullptr;
  v5 = result[2];
  v22 = result[1];
  v21 = 0;
  v23 = 0;
  v6 = (176 - v22) / 2;
  v24 = v5;
  v7 = (176 - v5) / 2;
  v25[1] = (v22 + 176) / 2 + 10;
  v25[3] = (v5 + 176) / 2 - 40;
  v25[0] = v6 + 10;
  v25[2] = v7 - 40;
  v8 = result[3];
  if ( v8 == 16 )
  {
    v9 = off_7BB78;
  }
  else
  {
    if ( v8 != 8 )
      goto LABEL_7;
    v9 = off_7BBB8;
  }
  v4 = v9[8];
LABEL_7:
  v10 = a1[2];
  a1[3] = v4;
  (*(void (__fastcall **)(int, _DWORD *, _DWORD *, int *, int (*)(), int *, int))(*(_DWORD *)v10 + 20))(
    v10,
    v25,
    v3,
    &v21,
    v4,
    &dword_81230,
    1);
  v11 = v6 + 15;
  v12 = (unsigned __int16 *)a1[184];
  v13 = v11;
  v14 = *v12;
  v15 = v7 - 35;
  result = (_DWORD *)(v14 << 16);
  v16 = v14;
  if ( *v12 != 0 )
  {
    do
    {
      if ( v16 == 65283 )
      {
        v15 += 14;
        v13 = v11;
      }
      else
      {
        v17 = a1[172];
        v18 = a1[2];
        v19 = sub_1B06C(v17, v14);
        sub_1B150(v17, v18, v13, v15, 12451, v19);
        v13 += 12;
      }
      v20 = v12[1];
      ++v12;
      v14 = v20;
      result = (_DWORD *)(v20 << 16);
      v16 = (unsigned __int16)v20;
    }
    while ( (_WORD)v20 != 0 );
  }
  return result;
}

