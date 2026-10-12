/* ea:   0x0006118C */
/* name: sub_6118C */
/* size: 80 */
int (__fastcall **__fastcall sub_6118C(HANDLE *a1))(void *)
{
  *a1 = off_7BC04;
  if ( (*((unsigned __int8 (__fastcall **)(HANDLE *))*a1 + 9))(a1) != 0 )
  {
    CloseHandle(a1[1]);
    a1[1] = (HANDLE)-1;
  }
  *a1 = &off_6D248;
  return &off_6D248;
}

