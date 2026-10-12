/* ea:   0x00063374 */
/* name: sub_63374 */
/* size: 68 */
int (__fastcall **__fastcall sub_63374(int a1))(void *)
{
  char v2; // r1
  int (__fastcall **result)(void *); // r0

  v2 = *(_BYTE *)(a1 + 4);
  *(_DWORD *)a1 = off_7BC84;
  if ( v2 != 0 )
    result = (int (__fastcall **)(void *))sub_634FC(a1);
  else
    result = &off_7BCA8;
  *(_DWORD *)a1 = &off_7BCA8;
  return result;
}

