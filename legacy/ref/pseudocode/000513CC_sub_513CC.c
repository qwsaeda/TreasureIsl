/* ea:   0x000513CC */
/* name: sub_513CC */
/* size: 608 */
int sub_513CC(_DWORD *a1, ...)
{
  int v2; // r5
  int v3; // r4
  _DWORD *v4; // r8
  int result; // r0
  int v6; // r6
  int v7; // r7
  __int16 v8; // r0
  int v9; // r0
  unsigned int v10; // r3
  int v11; // r4
  unsigned int v12; // r3
  unsigned int v13; // r2
  int v14; // r0
  bool v15; // zf
  int v16; // r0
  int v17; // r2
  int v18; // r5
  int v19; // r1
  _DWORD *v20; // r0
  int v21; // r0
  _DWORD v22[2]; // [sp+0h] [bp-54h] BYREF
  int v23; // [sp+8h] [bp-4Ch] BYREF
  int v24; // [sp+Ch] [bp-48h]
  int v25; // [sp+10h] [bp-44h]
  int v26; // [sp+14h] [bp-40h]
  int v27; // [sp+18h] [bp-3Ch]
  int v28; // [sp+1Ch] [bp-38h]
  int varg_r1; // [sp+48h] [bp-Ch] BYREF
  va_list varg_r1a; // [sp+48h] [bp-Ch]
  int varg_r2; // [sp+4Ch] [bp-8h]
  va_list varg_r3; // [sp+50h] [bp-4h] BYREF

  va_start(varg_r3, a1);
  va_start(varg_r1a, a1);
  varg_r1 = va_arg(varg_r3, _DWORD);
  varg_r2 = va_arg(varg_r3, _DWORD);
  v2 = varg_r2;
  v3 = *sub_1DE08((_DWORD *)(a1[6] + 508), v22);
  v4 = nullptr;
  result = *(_DWORD *)sub_52D7C(a1[6] + 508, v22, (int *)varg_r1a) == v3;
  if ( (_BYTE)result == 0 )
  {
    result = v2 << 16;
    if ( (_WORD)v2 != 0 )
    {
      result = sub_1623C((_DWORD *)(a1[6] + 508), (unsigned __int16 *)varg_r1a);
      v6 = *(_DWORD *)result;
      if ( *(_DWORD *)result != 0 )
      {
        v7 = *(_DWORD *)(v6 + 4);
        v8 = 999;
        if ( (unsigned __int16)v2 + (unsigned int)*(unsigned __int16 *)(v6 + 22) <= 0x3E7 )
          v8 = v2 + *(_WORD *)(v6 + 22);
        *(_WORD *)(v6 + 22) = v8;
        v9 = a1[2];
        if ( v9 <= 4 )
          *(_BYTE *)(v6 + 50) = v9;
        v10 = *(unsigned __int8 *)(v6 + 48);
        if ( v10 < 0xA )
        {
          ++*(_BYTE *)(v10 + a1[47]);
          ++a1[50];
        }
        if ( *(_BYTE *)(v6 + 49) == 0 )
        {
          v11 = a1[6] + 144;
          v12 = *(_DWORD *)(a1[6] + 156);
          v13 = *(_DWORD *)(a1[6] + 152) + 1;
          *(_DWORD *)(a1[6] + 152) = v13;
          if ( v13 / 7 >= v12 / 0xA )
            *(_DWORD *)(v11 + 28) = 1;
          sub_5350C(v6);
          v14 = *(_DWORD *)(v6 + 44);
          v23 = 0;
          v15 = v14 == 1;
          v16 = a1[6];
          v24 = 0;
          v25 = 0;
          v26 = 0;
          v27 = 0;
          v28 = 0;
          v17 = *(_DWORD *)(v16 + 32) + 4 * *(unsigned __int8 *)(v6 + 49);
          if ( v15 )
            v18 = *(_DWORD *)(v17 + 8);
          else
            v18 = *(_DWORD *)(v17 + 24);
          if ( v18 == 0 )
            __debugbreak();
          v19 = *(_DWORD *)(v6 + 16);
          v24 = *(_DWORD *)(v6 + 12) - *(_DWORD *)(v18 + 4) + 64;
          v25 = v19 - *(_DWORD *)(v18 + 8) + 32;
          v20 = operator new(0x14u);
          if ( v20 != nullptr )
            v4 = sub_430AC(v20);
          *(_DWORD *)(v6 + 8) = v4;
          if ( v4 != nullptr )
          {
            sub_11558(a1[6] + 64, v22, *(_DWORD *)(a1[6] + 64), (_DWORD *)(v6 + 8));
            v21 = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1[6] + 64) + 4) + 8);
            (*(void (__fastcall **)(int, int, int *, int))(*(_DWORD *)v21 + 8))(v21, v18, &v23, 1);
          }
          sub_3958C((_DWORD **)(a1[6] + 64));
        }
        return (*(int (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(v7 + 40) + 32))(*(_DWORD *)(v7 + 40), a1[15]);
      }
    }
  }
  return result;
}

