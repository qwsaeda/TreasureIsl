/* ea:   0x00043484 */
/* name: sub_43484 */
/* size: 1196 */
int __fastcall sub_43484(_DWORD *a1)
{
  int v2; // r3
  _DWORD *v3; // r2
  int v4; // r0
  int v5; // r1
  int (*v6)(); // r0
  int v7; // r4
  _DWORD *v8; // r7
  int v9; // r1
  int (*v10)(); // r6
  int v11; // r1
  int v12; // r0
  int v13; // r5
  int v14; // r2
  int v15; // r4
  _DWORD *v16; // r5
  int i; // r6
  int result; // r0
  int v19; // r1
  int (*v20)(); // r2
  int v21; // r4
  int v22; // r0
  int v23; // r0
  int v24; // r0
  int v25; // r0
  int v26; // r0
  _DWORD v27[4]; // [sp+Ch] [bp-84h] BYREF
  _DWORD v28[4]; // [sp+1Ch] [bp-74h] BYREF
  _DWORD v29[4]; // [sp+2Ch] [bp-64h] BYREF
  _DWORD v30[4]; // [sp+3Ch] [bp-54h] BYREF
  _DWORD v31[4]; // [sp+4Ch] [bp-44h] BYREF
  _DWORD v32[13]; // [sp+5Ch] [bp-34h] BYREF

  sub_443FC(a1);
  v2 = a1[258];
  if ( v2 == 4 )
  {
    v2 = 0;
    a1[257] = 0;
  }
  v3 = &a1[8 * v2];
  v4 = v3[260];
  if ( v4 == 0 )
    __debugbreak();
  v5 = *(_DWORD *)(v4 + 12);
  if ( v5 == 16 )
  {
    v6 = sub_5BC14;
  }
  else if ( v5 == 8 )
  {
    v6 = sub_5D9C4;
  }
  else
  {
    v6 = nullptr;
  }
  v27[0] = 0;
  v27[3] = 50;
  v28[2] = 65;
  v27[1] = 55;
  v27[2] = 6;
  v7 = a1[1];
  a1[122] = v6;
  v28[0] = 12;
  v28[1] = 67;
  v28[3] = 109;
  (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v7 + 20))(
    v7,
    v28,
    v3[260],
    v27,
    v6,
    &dword_81230,
    1);
  v8 = (_DWORD *)a1[256];
  if ( *v8 == 0 )
    __debugbreak();
  v9 = *(_DWORD *)(*v8 + 12);
  if ( v9 == 16 )
  {
    v10 = sub_5BC14;
  }
  else if ( v9 == 8 )
  {
    v10 = sub_5D9C4;
  }
  else
  {
    v10 = nullptr;
  }
  a1[122] = v10;
  v11 = v8[10];
  v29[1] = v8[9];
  v29[3] = v11;
  v29[0] = 0;
  v29[2] = 0;
  v12 = v8[10];
  v13 = v8[4];
  v14 = v8[3] + v8[9];
  v30[0] = v8[3];
  v15 = a1[1];
  v30[3] = v13 + v12;
  v30[1] = v14;
  v30[2] = v13;
  (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v15 + 20))(
    v15,
    v30,
    *v8,
    v29,
    v10,
    &dword_81230,
    1);
  sub_44714(a1, a1[8 * a1[257] + 261], 139, 63);
  sub_44714(a1, LOWORD(a1[8 * a1[257] + 263]), 115, 83);
  sub_44714(a1, LOWORD(a1[8 * a1[257] + 264]), 115, 103);
  sub_44714(a1, LOWORD(a1[8 * a1[257] + 265]), 169, 103);
  sub_44714(a1, HIWORD(a1[8 * a1[257] + 263]), 60, 133);
  sub_44714(a1, a1[8 * a1[257] + 262], 139, 133);
  sub_44714(a1, HIWORD(a1[8 * a1[257] + 264]), 60, 153);
  sub_44714(a1, BYTE2(a1[8 * a1[257] + 265]), 115, 153);
  sub_44714(a1, LOWORD(a1[8 * a1[257] + 266]), 153, 153);
  v16 = a1;
  for ( i = 3; i < 175; i += 43 )
  {
    result = v16[260];
    if ( result != 0 )
    {
      v19 = *(_DWORD *)(result + 12);
      if ( v19 == 16 )
      {
        v20 = sub_5BC14;
      }
      else if ( v19 == 8 )
      {
        v20 = sub_5D9C4;
      }
      else
      {
        v20 = nullptr;
      }
      v21 = a1[1];
      v31[1] = 48;
      v31[3] = 50;
      v32[1] = i + 40;
      v32[3] = 51;
      v31[2] = 6;
      a1[122] = v20;
      v32[2] = 7;
      v31[0] = 8;
      v32[0] = i;
      result = (*(int (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v21 + 20))(
                 v21,
                 v32,
                 v16[260],
                 v31,
                 v20,
                 &dword_81230,
                 1);
    }
    v16 += 8;
  }
  switch ( a1[258] )
  {
    case 0:
      v22 = a1[255];
      if ( v22 != 0 )
      {
        *(_DWORD *)(v22 + 20) = 1;
        a1[255] = 0;
      }
      result = sub_44648(a1, 31);
      break;
    case 1:
      v23 = a1[255];
      if ( v23 != 0 )
      {
        *(_DWORD *)(v23 + 20) = 1;
        a1[255] = 0;
      }
      result = sub_44648(a1, 75);
      break;
    case 2:
      v24 = a1[255];
      if ( v24 != 0 )
      {
        *(_DWORD *)(v24 + 20) = 1;
        a1[255] = 0;
      }
      result = sub_44648(a1, 120);
      break;
    case 3:
      v25 = a1[255];
      if ( v25 != 0 )
      {
        *(_DWORD *)(v25 + 20) = 1;
        a1[255] = 0;
      }
      result = sub_44648(a1, 160);
      break;
    case 4:
      v26 = a1[255];
      if ( v26 != 0 )
      {
        *(_DWORD *)(v26 + 20) = 1;
        a1[255] = 0;
      }
      result = sub_44648(a1, 140);
      break;
    default:
      return result;
  }
  return result;
}

