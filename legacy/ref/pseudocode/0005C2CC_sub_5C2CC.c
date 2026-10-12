/* ea:   0x0005C2CC */
/* name: sub_5C2CC */
/* size: 496 */
int __fastcall sub_5C2CC(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  int v8; // r0
  int v9; // r1
  bool v10; // zf
  int result; // r0
  int v12; // r0
  int v13; // r8
  int v14; // r5
  int v15; // r4
  int v16; // r3
  unsigned __int16 *v17; // r2
  int v18; // r6
  unsigned __int16 *v19; // r10
  unsigned __int16 *v20; // r9
  int v21; // r8
  int v22; // r5
  int v23; // [sp+0h] [bp-6Ch]
  unsigned __int16 *v24; // [sp+4h] [bp-68h]
  int v25; // [sp+8h] [bp-64h]
  _DWORD v28[4]; // [sp+18h] [bp-54h] BYREF
  _DWORD v29[4]; // [sp+28h] [bp-44h] BYREF
  int v30; // [sp+38h] [bp-34h]
  int v31; // [sp+3Ch] [bp-30h]
  int v32; // [sp+40h] [bp-2Ch]
  int v33; // [sp+44h] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == 0 || *(_DWORD *)(a3 + 12) != 16 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v8 = a1[1];
  v9 = a1[2];
  v28[0] = 0;
  v28[1] = v8;
  v28[2] = 0;
  v28[3] = v9;
  sub_5E9E0(v29, v28, a2);
  v10 = v30 == v31;
  if ( v30 == v31 )
    v10 = v33 == v32;
  LOBYTE(result) = v10;
  result = (unsigned __int8)result;
  if ( (_BYTE)result == 0 )
  {
    v12 = sub_5E970(a1, v32);
    v13 = v12 + 2 * v30;
    v14 = v33 - v32;
    v23 = v13;
    v25 = v33 - v32;
    v15 = v31 - v30;
    if ( a3 == 0 )
      __debugbreak();
    v16 = sub_5E970(a3, v29[2]);
    result = v29[0];
    v17 = (unsigned __int16 *)(v16 + 2 * v29[0]);
    v24 = v17;
    if ( v14 != 0 )
    {
      v18 = v15;
      while ( 1 )
      {
        v19 = &v17[v18];
        v20 = v17;
        if ( v17 != &v17[v18] )
        {
          v21 = v13 - (_DWORD)v17;
          v22 = (unsigned __int16)word_8122C;
          do
          {
            if ( v22 != *v20 )
              *(unsigned __int16 *)((char *)v20 + v21) = (*(int (__fastcall **)(int, _DWORD, _DWORD))(*(_DWORD *)a5 + 8))(
                                                           a5,
                                                           *v20,
                                                           *(unsigned __int16 *)((char *)v20 + v21));
            ++v20;
          }
          while ( v20 != v19 );
          v17 = v24;
          v14 = v25;
        }
        v23 += 2 * a1[1];
        if ( a3 == 0 )
          __debugbreak();
        result = *(_DWORD *)(a3 + 4);
        v25 = --v14;
        v17 += result;
        v24 = v17;
        if ( v14 == 0 )
          break;
        v13 = v23;
      }
    }
  }
  return result;
}

