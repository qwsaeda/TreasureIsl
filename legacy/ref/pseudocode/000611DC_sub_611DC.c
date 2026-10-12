/* ea:   0x000611DC */
/* name: sub_611DC */
/* size: 52 */
int __fastcall sub_611DC(HANDLE *a1)
{
  int result; // r0

  result = (*((int (__fastcall **)(HANDLE *))*a1 + 9))(a1);
  if ( (_BYTE)result != 0 )
  {
    result = CloseHandle(a1[1]);
    a1[1] = (HANDLE)-1;
  }
  return result;
}

