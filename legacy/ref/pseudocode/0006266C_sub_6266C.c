/* ea:   0x0006266C */
/* name: sub_6266C */
/* size: 40 */
int __fastcall sub_6266C(int a1)
{
  int result; // r0

  result = *(_DWORD *)(a1 + 8);
  if ( result != 0 )
    return (*(int (__fastcall **)(int))(*(_DWORD *)result + 24))(result);
  return result;
}

