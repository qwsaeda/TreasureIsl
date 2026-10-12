/* ea:   0x0002EE34 */
/* name: sub_2EE34 */
/* size: 692 */
int sub_2EE34(int result, ...)
{
  _DWORD *v1; // r8
  _DWORD *i; // r6
  int v3; // r0
  _DWORD **v4; // r0
  void *v5; // r4
  _DWORD *j; // r6
  int v7; // r0
  void (__fastcall ***v8)(_DWORD, int); // r3
  _DWORD *v9; // r0
  _DWORD *v10; // r0
  int v11; // [sp+0h] [bp-40h] BYREF
  int v12; // [sp+4h] [bp-3Ch] BYREF
  _DWORD v13[10]; // [sp+8h] [bp-38h] BYREF
  int varg_r1; // [sp+34h] [bp-Ch] BYREF
  va_list varg_r1a; // [sp+34h] [bp-Ch]
  va_list varg_r2; // [sp+38h] [bp-8h] BYREF

  va_start(varg_r2, result);
  va_start(varg_r1a, result);
  varg_r1 = va_arg(varg_r2, _DWORD);
  v1 = (_DWORD *)result;
  if ( varg_r1 != 1 )
  {
    if ( varg_r1 <= 4 )
    {
      *(_DWORD *)(result + 188) += 25;
      for ( i = (_DWORD *)*sub_1DDF0(result + 508, v13); i != (_DWORD *)*sub_1DE08(v1 + 127, v13); i = sub_16A30(i) )
      {
        v3 = i[5];
        if ( *(unsigned __int8 *)(v3 + 50) == varg_r1 )
        {
          (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(*(_DWORD *)(v3 + 4) + 40) + 32))(
            *(_DWORD *)(*(_DWORD *)(v3 + 4) + 40),
            *(_DWORD *)(v1[8] + 8));
          *(_BYTE *)(i[5] + 50) = 0;
        }
      }
      sub_2F944(v1 + 130, (int *)varg_r1a);
      v4 = (_DWORD **)sub_2F944(v1 + 130, (int *)varg_r1a);
      v5 = *v4;
      if ( *v4 != nullptr )
      {
        sub_1B2AC(*v4);
        operator delete(v5);
      }
      sub_30DB0(v1 + 130, (int *)varg_r1a);
      sub_30EB8(v1 + 133, (int *)varg_r1a);
      for ( j = *(_DWORD **)sub_30C2C(v1 + 124, v13); j != (_DWORD *)*sub_1DE18(v1 + 124, v13); j = sub_16A30(j) )
      {
        if ( (int)j[4] > 4 )
        {
          v7 = (*(int (__fastcall **)(_DWORD))(*(_DWORD *)j[5] + 136))(j[5]);
          if ( v7 == varg_r1 )
          {
            v11 = j[5];
            sub_2F7B0(v1 + 16, &v11);
            *(_BYTE *)(j[5] + 4) = 0;
            (*(void (__fastcall **)(_DWORD))(*(_DWORD *)j[5] + 16))(j[5]);
            v8 = (void (__fastcall ***)(_DWORD, int))j[5];
            if ( v8 != nullptr )
              (**v8)(j[5], 1);
            sub_30C44(v1 + 124, j);
            sub_30EB8(v1 + 133, j + 4);
          }
        }
      }
    }
    v12 = *(_DWORD *)sub_1DD10(v1 + 124, (int *)varg_r1a);
    sub_2F7B0(v1 + 16, &v12);
    *(_BYTE *)(*(_DWORD *)sub_1DD10(v1 + 124, (int *)varg_r1a) + 4) = 0;
    v9 = (_DWORD *)sub_1DD10(v1 + 124, (int *)varg_r1a);
    (*(void (__fastcall **)(_DWORD))(*(_DWORD *)*v9 + 16))(*v9);
    v10 = (_DWORD *)sub_1DD10(v1 + 124, (int *)varg_r1a);
    if ( *v10 != 0 )
      (**(void (__fastcall ***)(_DWORD, int))*v10)(*v10, 1);
    return sub_30C80(v1 + 124, (int *)varg_r1a);
  }
  return result;
}

