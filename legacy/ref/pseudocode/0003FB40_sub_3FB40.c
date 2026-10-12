/* ea:   0x0003FB40 */
/* name: sub_3FB40 */
/* size: 692 */
int __fastcall sub_3FB40(_DWORD *a1)
{
  unsigned __int16 *v2; // r9
  int v3; // r8
  int v4; // r7
  int v5; // r2
  int v6; // r6
  int v7; // r1
  unsigned int v8; // r5
  int v9; // r4
  int v10; // r0
  int v11; // t1
  int v12; // r1
  int result; // r0
  _DWORD *v14; // r5
  int v15; // r0
  int (*v16)(); // r1
  int v17; // r0
  int v18; // r0
  int v19; // r0
  int (*v20)(); // r3
  int v21; // r0
  int v22; // r4
  unsigned int v23; // r0
  int v24; // r5
  int v25; // r3
  unsigned int v26; // r0
  unsigned int v27; // r3
  int v28; // r4
  int v29; // r2
  int v30; // r0
  int v31; // [sp+Ch] [bp-44h] BYREF
  int v32; // [sp+10h] [bp-40h]
  int v33; // [sp+14h] [bp-3Ch]
  int v34; // [sp+18h] [bp-38h]
  int v35; // [sp+1Ch] [bp-34h] BYREF
  int v36; // [sp+20h] [bp-30h]
  int v37; // [sp+24h] [bp-2Ch]
  int v38; // [sp+28h] [bp-28h]

  sub_443FC(a1);
  v2 = (unsigned __int16 *)a1[a1[256] + 514];
  v3 = 4;
  v4 = 25;
  v5 = *v2;
  v6 = 0;
  v7 = v5;
  if ( *v2 != 0 )
  {
    do
    {
      if ( v6 >= 11 )
        break;
      if ( v7 == 65283 )
      {
        v3 = 4;
        v4 += 13;
        ++v6;
      }
      else
      {
        v8 = a1[255];
        v9 = a1[1];
        v10 = sub_1B06C(v8, v5);
        sub_1B150(v8, v9, v3, v4, 12451, v10);
        v3 += 12;
      }
      v11 = v2[1];
      ++v2;
      v5 = v11;
      v7 = (unsigned __int16)v11;
    }
    while ( (_WORD)v11 != 0 );
  }
  v12 = a1[256];
  if ( v12 < a1[257] )
    a1[v12 + 515] = v2;
  result = a1[522];
  v14 = (_DWORD *)result;
  if ( result != 0 )
  {
    result = a1[523];
    a1[523] = result + 1;
    if ( result == 23 )
    {
      a1[523] = 0;
    }
    else if ( result + 1 >= 12 )
    {
      return result;
    }
    v15 = v14[3];
    if ( v15 == 16 )
    {
      v16 = sub_5BC14;
    }
    else if ( v15 == 8 )
    {
      v16 = sub_5D9C4;
    }
    else
    {
      v16 = nullptr;
    }
    v17 = a1[256];
    a1[122] = v16;
    if ( v17 > 0 )
    {
      v18 = v14[1];
      v31 = 0;
      v32 = v18;
      v33 = 0;
      v34 = v14[2] >> 1;
      v19 = v14[3];
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
      a1[122] = v20;
      v21 = v14[1];
      v36 = 90;
      v22 = a1[1];
      v35 = 90 - v21;
      v23 = v14[2];
      v38 = 20;
      v37 = 20 - (v23 >> 1);
      (*(void (__fastcall **)(int, int *, _DWORD *, int *, int (*)(), int *, int))(*(_DWORD *)v22 + 20))(
        v22,
        &v35,
        v14,
        &v31,
        v20,
        &dword_81230,
        1);
    }
    result = a1[257];
    if ( a1[256] < result )
    {
      v24 = a1[522];
      if ( v24 == 0 )
        __debugbreak();
      v25 = *(_DWORD *)(a1[522] + 8);
      if ( v24 == 0 )
      {
        __debugbreak();
        __debugbreak();
      }
      v36 = *(_DWORD *)(v24 + 4);
      v35 = 0;
      v26 = *(_DWORD *)(v24 + 8);
      v38 = v25;
      v37 = v26 >> 1;
      if ( v24 == 0 )
        __debugbreak();
      v27 = *(_DWORD *)(v24 + 8);
      if ( v24 == 0 )
        __debugbreak();
      v28 = a1[1];
      v29 = 90 - *(_DWORD *)(v24 + 4);
      v34 = 180;
      v31 = v29;
      v33 = 180 - (v27 >> 1);
      v30 = a1[122];
      v32 = 90;
      return (*(int (__fastcall **)(int, int *, int, int *, int, int *, int))(*(_DWORD *)v28 + 20))(
               v28,
               &v31,
               v24,
               &v35,
               v30,
               &dword_81230,
               1);
    }
  }
  return result;
}

