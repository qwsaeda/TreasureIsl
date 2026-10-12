/* ea:   0x0005CFF0 */
/* name: sub_5CFF0 */
/* size: 544 */
int __fastcall sub_5CFF0(_DWORD *a1, int a2, _DWORD *a3, int a4, int a5)
{
  int v7; // r1
  unsigned __int8 v8; // r4
  bool v9; // zf
  int result; // r0
  int v11; // r0
  _WORD *v12; // r6
  int v13; // r5
  int v14; // r4
  int v15; // r9
  int v16; // r10
  int v17; // r11
  _WORD *v18; // r5
  int i; // r6
  int v20; // t1
  int v21; // r5
  int j; // r7
  _WORD *v23; // [sp+4h] [bp-6Ch]
  int v24; // [sp+8h] [bp-68h]
  int v25; // [sp+Ch] [bp-64h]
  _DWORD v28[4]; // [sp+1Ch] [bp-54h] BYREF
  _DWORD v29[4]; // [sp+2Ch] [bp-44h] BYREF
  int v30; // [sp+3Ch] [bp-34h]
  int v31; // [sp+40h] [bp-30h]
  int v32; // [sp+44h] [bp-2Ch]
  int v33; // [sp+48h] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == nullptr || a3[3] != 8 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v7 = a1[2];
  v8 = 0;
  v28[1] = a1[1];
  v28[3] = v7;
  v28[0] = 0;
  v28[2] = 0;
  sub_5E9E0(v29, v28, a2);
  v9 = v30 == v31;
  if ( v30 == v31 )
    v9 = v33 == v32;
  if ( v9 )
    v8 = 1;
  result = v8;
  if ( v8 == 0 )
  {
    v11 = sub_5E970(a1, v32);
    v12 = (_WORD *)(v11 + 2 * v30);
    v13 = v33 - v32;
    v23 = v12;
    v25 = v33 - v32;
    v14 = v31 - v30;
    v24 = sub_5E970(a3, v29[2]);
    if ( a3 == nullptr )
      __debugbreak();
    v15 = a3[8];
    v16 = a3[1] - v29[0];
    v17 = v16 - v14;
    result = sub_5E920(a3);
    if ( (_BYTE)result != 0 )
    {
      if ( v13 != 0 )
      {
        while ( 1 )
        {
          v18 = v12;
          for ( i = v16 + v24; i != v17 + v24; ++v18 )
          {
            v20 = *(unsigned __int8 *)--i;
            *v18 = *(_WORD *)(v15
                            + 2
                            * (*(unsigned __int8 (__fastcall **)(int, int, int, _DWORD))(*(_DWORD *)a5 + 4))(
                                a5,
                                v20,
                                v15,
                                (unsigned __int16)*v18));
          }
          result = (int)a3;
          v23 += a1[1];
          --v25;
          v24 += a3[1];
          if ( v25 == 0 )
            break;
          v12 = v23;
        }
      }
    }
    else
    {
      v21 = v25;
      for ( j = v24; v21 != 0; j += result )
      {
        sub_5EC44(v17 + j, v16 + j, v12, v15, a5);
        --v21;
        result = a3[1];
        v12 += a1[1];
      }
    }
  }
  return result;
}

