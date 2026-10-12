/* ea:   0x0003E758 */
/* name: sub_3E758 */
/* size: 372 */
int *__fastcall sub_3E758(int a1, _WORD *a2)
{
  int v4; // r6
  _DWORD *v5; // r1
  int v6; // r4
  int *result; // r0
  int *v8; // r5
  int v9; // r0

  v4 = sub_17FD8(a2);
  v5 = *(_DWORD **)(a1 + 12);
  if ( v5 != nullptr )
  {
    if ( *v5 == v4 )
      return (int *)sub_598BC(*(_DWORD *)(a1 + 20), v5 + 2);
    sub_3E8CC(a1);
    v6 = (*(int (__fastcall **)(_DWORD, _WORD *, _DWORD, _DWORD))(**(_DWORD **)(a1 + 16) + 8))(
           *(_DWORD *)(a1 + 16),
           a2,
           0,
           0);
    if ( v6 == 0 )
      return (int *)sub_598BC(*(_DWORD *)(a1 + 20), *(_DWORD *)(a1 + 12) + 8);
    result = (int *)operator new(0x28u);
    v8 = result;
    if ( result != nullptr )
    {
      result[1] = 0;
      result = (int *)sub_5870C(result + 2);
    }
    else
    {
      v8 = nullptr;
    }
    *(_DWORD *)(a1 + 12) = v8;
    if ( v8 != nullptr )
      goto LABEL_15;
  }
  else
  {
    v6 = (*(int (__fastcall **)(_DWORD, _WORD *, _DWORD, _DWORD))(**(_DWORD **)(a1 + 16) + 8))(
           *(_DWORD *)(a1 + 16),
           a2,
           0,
           0);
    if ( v6 == 0 )
      return (int *)sub_598BC(*(_DWORD *)(a1 + 20), *(_DWORD *)(a1 + 12) + 8);
    result = (int *)operator new(0x28u);
    v8 = result;
    if ( result != nullptr )
    {
      result[1] = 0;
      result = (int *)sub_5870C(result + 2);
    }
    else
    {
      v8 = nullptr;
    }
    *(_DWORD *)(a1 + 12) = v8;
    if ( v8 != nullptr )
    {
LABEL_15:
      *v8 = v4;
      v8[1] = v6;
      v9 = (*(int (__fastcall **)(int))(*(_DWORD *)v6 + 12))(v6);
      sub_587D8(v8 + 2, v9);
      sub_587D0(*(_DWORD *)(a1 + 12) + 8, 1);
      return (int *)sub_598BC(*(_DWORD *)(a1 + 20), *(_DWORD *)(a1 + 12) + 8);
    }
  }
  return result;
}

